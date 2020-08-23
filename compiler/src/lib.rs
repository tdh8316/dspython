use std::fs::read_to_string;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::passes::{PassManager, PassManagerBuilder};
use inkwell::support::LLVMString;
use inkwell::targets::{TargetData, TargetTriple};
use inkwell::OptimizationLevel;

use dsp_compiler_error::{LLVMCompileError, LLVMCompileErrorType};
use dsp_python_codegen::cgexpr::CGExpr;
use dsp_python_codegen::cgstmt::CGStmt;
use dsp_python_codegen::CodeGen;
use dsp_python_parser::parser::parse_program;
use dsp_python_parser::{ast, CompileError};

pub use crate::flags::*;
use crate::llvm_prototypes::generate_prototypes;

pub mod flags;
mod llvm_prototypes;

type CompileResult<T> = Result<T, LLVMCompileError>;

pub struct Compiler<'a, 'ctx> {
    pub source_path: String,
    pub compiler_flags: CompilerFlags,

    cg: CodeGen<'a, 'ctx>,
    pass_manager: PassManager<Module<'ctx>>,
}

impl<'a, 'ctx> Compiler<'a, 'ctx> {
    pub fn new(
        source_path: String,
        compiler_flags: CompilerFlags,
        context: &'ctx Context,
        builder: &'a Builder<'ctx>,
        module: &'a Module<'ctx>,
        pass_manager: PassManager<Module<'ctx>>,
    ) -> Self {
        Compiler {
            source_path,
            compiler_flags,
            cg: CodeGen::new(context, builder, module),
            pass_manager,
        }
    }

    pub fn compile(&mut self, program: ast::Program) -> CompileResult<()> {
        generate_prototypes(self.cg.module, self.cg.context);
        for statement in program.statements.iter() {
            if let ast::StatementType::Expression { ref expression } = statement.node {
                self.cg.compile_expr(&expression)?;
            } else {
                self.cg.compile_stmt(&statement)?;
            }
        }
        Ok(())
    }

    pub fn emit(&self) -> LLVMString {
        self.pass_manager.run_on(&self.cg.module);
        self.cg.module.print_to_string()
    }
}

pub fn compile(source_path: String, flags: CompilerFlags) -> CompileResult<LLVMString> {
    let to_compile_error =
        |parse_error| CompileError::from_parse_error(parse_error, source_path.clone());
    let source = read_to_string(&source_path).expect(&format!(
        "python: can't open file '{}': [Errno 2] No such file or directory",
        source_path
    ));
    let ast = parse_program(&source)
        .map_err(to_compile_error)
        .expect(&format!(
            "Failed to parse '{}' because of error above.",
            source_path
        ));

    let context = Context::create();
    let module = context.create_module(&source_path);
    // Create target data structure for Arduino
    let target_data = TargetData::create("e-P1-p:16:8-i8:8-i16:8-i32:8-i64:8-f32:8-f64:8-n8-a:8");
    module.set_data_layout(&target_data.get_data_layout());
    // LLVM triple
    module.set_triple(&TargetTriple::create("avr"));
    // Create a root builder context
    let builder = context.create_builder();

    // Initialize pass manager
    let pass_manager: PassManager<Module> = PassManager::create(());
    let pm_builder = PassManagerBuilder::create();
    pm_builder.set_optimization_level(match flags.optimization_level {
        0 => OptimizationLevel::None,
        1 => OptimizationLevel::Less,
        2 => OptimizationLevel::Default,
        3 => OptimizationLevel::Aggressive,
        _ => {
            return Err(LLVMCompileError::new(
                None,
                LLVMCompileErrorType::NotImplemented(
                    "Optimization level must be a integer of 0~3".to_string(),
                ),
            ));
        }
    });
    pm_builder.populate_module_pass_manager(&pass_manager);

    let mut compiler = Compiler::new(
        source_path,
        flags,
        &context,
        &builder,
        &module,
        pass_manager,
    );

    compiler.compile(ast)?;

    Ok(compiler.emit())
}