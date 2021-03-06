def analog_input_to_digital_pin(p: int) -> int:
    if p < 6:
        return p + 14
    else:
        return -1


def digital_pin_to_interrupt(p: int) -> int:
    if p == 2:
        return 0
    elif p == 3:
        return 1

    # Not an interrupt
    return -1


SS = 10
MOSI = 11
MISO = 12
SCK = 13

SDA = 18
SCL = 19

LED_BUILTIN = 13

A0 = 14
A1 = 15
A2 = 16
A3 = 17
A4 = 18
A5 = 19
A6 = 20
A7 = 21
