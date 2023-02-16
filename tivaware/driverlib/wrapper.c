//
// wrapper.h - Small wrapper around driverlib for eCTF. This only exposes what
// we need (EEPROM, UART, and GPIO) and provides a few convenience functions.
//

#include <stdbool.h>
#include <stdint.h>

#include "driverlib/wrapper.h"

#include "driverlib/eeprom.c"
#include "driverlib/gpio.c"
#include "driverlib/pin_map.h"
#include "driverlib/uart.c"
#include "driverlib/sysctl.c"

#define HOST_UART ((uint32_t)UART0_BASE)
#define BOARD_UART ((uint32_t)UART1_BASE)

/**
 * @brief Initialize the UART interfaces.
 *
 * UART 0 is used to communicate with the host computer.
 */
static void uart_init(void) {
  // Configure the UART peripherals used in this example
  // RCGC   Run Mode Clock Gating
  SysCtlPeripheralEnable(SYSCTL_PERIPH_UART0); // UART 0 for host interface
  SysCtlPeripheralEnable(SYSCTL_PERIPH_GPIOA); // UART 0 is on GPIO Port A
  // HBCTL  High-performance Bus Control
  // PCTL   Port Control
  GPIOPinConfigure(GPIO_PA0_U0RX);
  GPIOPinConfigure(GPIO_PA1_U0TX);
  // DIR    Direction
  // AFSEL  Alternate Function Select
  // DR2R   2-mA Drive Select
  // DR4R   4-mA Drive Select
  // DR8R   8-mA Drive Select
  // SLR    Slew Rate Control Select
  // ODR    Open Drain Select
  // PUR    Pull-Up Select
  // PDR    Pull-Down Select
  // DEN    Digital Enable
  // AMSEL  Analog Mode Select
  GPIOPinTypeUART(GPIO_PORTA_BASE, GPIO_PIN_0 | GPIO_PIN_1);

  // Configure the UART for 115,200, 8-N-1 operation.
  UARTConfigSetExpClk(
      UART0_BASE, SysCtlClockGet(), 115200,
      (UART_CONFIG_WLEN_8 | UART_CONFIG_STOP_ONE | UART_CONFIG_PAR_NONE));
}

/**
 * @brief Set the up board link object
 *
 * UART 1 is used to communicate between boards
 */
static void setup_board_link(void) {
  SysCtlPeripheralEnable(SYSCTL_PERIPH_UART1);
  SysCtlPeripheralEnable(SYSCTL_PERIPH_GPIOB);

  GPIOPinConfigure(GPIO_PB0_U1RX);
  GPIOPinConfigure(GPIO_PB1_U1TX);

  GPIOPinTypeUART(GPIO_PORTB_BASE, GPIO_PIN_0 | GPIO_PIN_1);

  // Configure the UART for 115,200, 8-N-1 operation.
  UARTConfigSetExpClk(
      BOARD_UART, SysCtlClockGet(), 115200,
      (UART_CONFIG_WLEN_8 | UART_CONFIG_STOP_ONE | UART_CONFIG_PAR_NONE));

  while (UARTCharsAvail(BOARD_UART)) {
    UARTCharGet(BOARD_UART);
  }
}

void init_system(void) {
  // Ensure EEPROM peripheral is enabled
  SysCtlPeripheralEnable(SYSCTL_PERIPH_EEPROM0);
  EEPROMInit();

  // Initialize UART peripheral
  uart_init();

  // Initialize board link UART
  setup_board_link();
}

bool uart_avail_host(void) { return UARTCharsAvail(HOST_UART); }
bool uart_avail_board(void) { return UARTCharsAvail(BOARD_UART); }
int32_t uart_readb_host(void) { return UARTCharGet(HOST_UART); }
int32_t uart_readb_board(void) { return UARTCharGet(BOARD_UART); }
void uart_writeb_host(uint8_t data) { UARTCharPut(HOST_UART, data); }
void uart_writeb_board(uint8_t data) { UARTCharPut(BOARD_UART, data); }

void eeprom_read(uint32_t *data, uint32_t address, uint32_t count) { EEPROMRead(data, address, count); }
void eeprom_write(uint32_t *data, uint32_t address, uint32_t count) { EEPROMProgram(data, address, count); }

bool read_sw_1(void) {
  return GPIOPinRead(GPIO_PORTF_BASE, GPIO_PIN_4) == 0;
}
