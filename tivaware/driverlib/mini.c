//
// mini.h - Small version of driverlib for eCTF.
//

#include <stdbool.h>
#include <stdint.h>

#include "driverlib/mini.h"

#include "driverlib/eeprom.c"
#include "driverlib/sysctl.c"

void
InitSystem(void)
{
  // Ensure EEPROM peripheral is enabled
  SysCtlPeripheralEnable(SYSCTL_PERIPH_EEPROM0);
  EEPROMInit();

  // Initialize UART peripheral
  /* uart_init(); */

  // Initialize board link UART
  /* setup_board_link(); */
}
