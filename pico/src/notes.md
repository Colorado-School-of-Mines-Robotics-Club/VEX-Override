# PCB mappings

J13 -> I2C0 (GP8)
Otos -> I2C0 (GP8)
J5-J12 -> Multiplexered in order
J15 (unlabled) -> I2C multiplexer bypass (I2C1 on GP10)
Rs485 -> GP1 GP0 UART0
J14 (RGBs) -> [5v, GP6, GP7, GND]
Rs485 en -> GP14
Top LED -> GP2
Bottom LED -> GP3
SW1 -> RUN (reset button)
SW2 -> GP12
Lidar -> GP5 & GP4 (UART1)

## UART protocol

Pico initiates all messages

### Pico -> brain messages

1. `l`: Lidar measurements (any number of `[angle f32][distance f32][quality u8]`)
1. `o`: OTOS measurements (raw i2c register reads starting with status and ending with accel std. dev.)
1. `r`: Request updates from brain (brain response format below)

### Brain -> pico messages

1. `c`: Calibrate OTOS
1. `p`: Set OTOS position
1. `s`: Set OTOS scalars & offsets (`[offset][scalars]`)
