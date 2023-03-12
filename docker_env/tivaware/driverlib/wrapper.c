//
// wrapper.h - Small wrapper around driverlib for eCTF. This only exposes what
// we need (EEPROM, UART, and GPIO) and provides a few convenience functions.
//

#include <stdbool.h>
#include <stdint.h>

#include "driverlib/wrapper.h"

#include "driverlib/adc.c"
#include "driverlib/eeprom.c"
#include "driverlib/gpio.c"
#include "driverlib/pin_map.h"
#include "driverlib/uart.c"
#include "driverlib/sysctl.c"
#include "driverlib/timer.c"

#include "inc/tm4c123gh6pm.h"

#define HOST_UART ((uint32_t)UART0_BASE)
#define BOARD_UART ((uint32_t)UART1_BASE)

#define TEMP_SAMPLES 8

/**
 * @brief Initialize the UART interfaces.
 *
 * UART 0 is used to communicate with the host computer.
 */
static void uart_init(void) {
  SysCtlPeripheralEnable(SYSCTL_PERIPH_UART0); // UART 0 for host interface
  SysCtlPeripheralEnable(SYSCTL_PERIPH_GPIOA); // UART 0 is on GPIO Port A
  GPIOPinConfigure(GPIO_PA0_U0RX);
  GPIOPinConfigure(GPIO_PA1_U0TX);
  GPIOPinTypeUART(GPIO_PORTA_BASE, GPIO_PIN_0 | GPIO_PIN_1);

  // Configure the UART for 115,200, 8-N-1 operation.
  UARTConfigSetExpClk(
      UART0_BASE, SysCtlClockGet(), 115200,
      (UART_CONFIG_WLEN_8 | UART_CONFIG_STOP_ONE | UART_CONFIG_PAR_NONE));
}

/**
 * @brief Set the up board link object.
 *
 * UART 1 is used to communicate between boards.
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

/**
 * @brief Initialize the ADC for temperature sampling.
 * 
 * Temperature sensor is one factor used for entropy generation.
 */
static void adc_init(void) {
  SysCtlPeripheralEnable(SYSCTL_PERIPH_ADC0);
  while (!SysCtlPeripheralReady(SYSCTL_PERIPH_ADC0)) {}
  // Disable oversample to increase noise
  ADCHardwareOversampleConfigure(ADC0_BASE, 0);
  ADCSequenceDisable(ADC0_BASE, 0);
  ADCSequenceConfigure(ADC0_BASE, 0, ADC_TRIGGER_PROCESSOR, 0);
  // Sample TEMP_SAMPLES samples and interrupt on last sample
  for (int i = 0; i < TEMP_SAMPLES - 1; ++i) {
    ADCSequenceStepConfigure(ADC0_BASE, 0, i, ADC_CTL_TS | ADC_CTL_SHOLD_4);
  }
  ADCSequenceStepConfigure(ADC0_BASE, 0, TEMP_SAMPLES - 1, ADC_CTL_TS | ADC_CTL_IE | ADC_CTL_END | ADC_CTL_SHOLD_4);
  ADCSequenceEnable(ADC0_BASE, 0);
}

/**
 * @brief Initialize the delay timer.
 * 
 * The delay timer is used to create arbitrary delays in the program.
 */
static void delay_timer_init(void) {
  SysCtlPeripheralEnable(SYSCTL_PERIPH_TIMER0);
  while (!SysCtlPeripheralReady(SYSCTL_PERIPH_TIMER0)) {}
  TimerConfigure(TIMER0_BASE, TIMER_CFG_ONE_SHOT);
  TimerClockSourceSet(TIMER0_BASE, TIMER_CLOCK_SYSTEM);
  TimerIntEnable(TIMER0_BASE, TIMER_TIMA_TIMEOUT);
}

/**
 * @brief Initialize the tick timer.
 * 
 * The tick timer is used to keep track of the current system time and is 
 * used for entropy generation.
 */
static void tick_timer_init(void) {
  SysCtlPeripheralEnable(SYSCTL_PERIPH_WTIMER0);
  while (!SysCtlPeripheralReady(SYSCTL_PERIPH_WTIMER0)) {}
  TimerConfigure(WTIMER0_BASE, TIMER_CFG_PERIODIC_UP);
  TimerClockSourceSet(WTIMER0_BASE, TIMER_CLOCK_PIOSC);
  TimerEnable(WTIMER0_BASE, TIMER_A);
}

/**
 * @brief Initialize the system.
 * 
 * This function initializes the system peripherals and sets up the board link.
 */
void init_system(void) {
  // Set the clocking to run directly from the crystal.
  SysCtlClockSet(SYSCTL_SYSDIV_2_5 | SYSCTL_USE_PLL | SYSCTL_OSC_INT);

  // Initialize the ADC temperature sensor
  adc_init();

  // Initialize the delay timer
  delay_timer_init();

  // Initialize the tick timer
  tick_timer_init();

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

/**
 * @brief Sample the temperature sensor.
 * 
 * @param samples The array to store the samples in.
 */
void get_temp_samples(uint32_t* samples) {
  ADCProcessorTrigger(ADC0_BASE, 0);
  while (!ADCIntStatus(ADC0_BASE, 0, false)) {}
  ADCIntClear(ADC0_BASE, 0);
  ADCSequenceDataGet(ADC0_BASE, 0, samples);
}

/**
 * @brief Sleep for a given number of microseconds. Program execution will
 *        resume after the given number of microseconds.
 * 
 * @param us The number of microseconds to sleep for.
 */
void sleep_us(uint32_t us) {
  uint32_t cycles = ((uint64_t)(us) * (uint64_t)(SysCtlClockGet())) / 3 / 1e6;
  SysCtlDelay(cycles);
}

/**
 * @brief Start a delay timer for a given number of microseconds. Program 
 *        execution will continue after the timer has been started.
 * 
 * @param us The number of microseconds to delay for.
 */
void start_delay_timer_us(uint32_t us) {
  uint32_t cycles = ((uint64_t)(us) * (uint64_t)(SysCtlClockGet())) / 1e6;
  TimerIntClear(TIMER0_BASE, TIMER_TIMA_TIMEOUT);
  TimerLoadSet(TIMER0_BASE, TIMER_A, cycles);
  TimerEnable(TIMER0_BASE, TIMER_A);
}

/**
 * @brief Wait for the delay timer to finish, similar to sleep_us.
 * 
 */
void wait_delay_timer(void) {
  while(!TimerIntStatus(TIMER0_BASE, false)) {}
  TimerIntClear(TIMER0_BASE, TIMER_TIMA_TIMEOUT);
}

// Based on TimerEnable
bool IsTimerEnabled(uint32_t ui32Base, uint32_t ui32Timer) {
    // Check the arguments.
    ASSERT(_TimerBaseValid(ui32Base));
    ASSERT((ui32Timer == TIMER_A) || (ui32Timer == TIMER_B) ||
           (ui32Timer == TIMER_BOTH));

    // Check if the timer modules are enabled.
    return (HWREG(ui32Base + TIMER_O_CTL) & (ui32Timer & (TIMER_CTL_TAEN |
                                                  TIMER_CTL_TBEN))) != 0;
}

/**
 * @brief Get the remaining time on the delay timer.
 * 
 * @return The remaining time on the delay timer in microseconds.
 */
uint32_t get_remaining_us_delay_timer(void) {
  if (IsTimerEnabled(TIMER0_BASE, TIMER_A)) {
    uint32_t curr_timer = TimerValueGet(TIMER0_BASE, TIMER_A);
    return ((uint64_t)(curr_timer) * 1e6) / ((uint64_t)(SysCtlClockGet()));
  } else {
    return 0;
  }
}

/**
 * @brief Get the current tick timer value.
 * 
 * @return The current tick timer value.
 */
uint64_t get_tick_timer(void) {
  return TimerValueGet64(WTIMER0_BASE);
}
