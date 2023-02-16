//
// wrapper.h - Small version of driverlib for eCTF.
//

#ifndef WRAPPER_H_
#define WRAPPER_H_

#include <stdbool.h>
#include <stdint.h>

extern void init_system(void);
extern bool uart_avail_host(void);
extern bool uart_avail_board(void);
extern int32_t uart_readb_host(void);
extern int32_t uart_readb_board(void);
extern void uart_writeb_host(uint8_t data);
extern void uart_writeb_board(uint8_t data);
extern void eeprom_read(uint32_t *data, uint32_t address, uint32_t count);
extern void eeprom_write(uint32_t *data, uint32_t address, uint32_t count);
extern bool check_switch(void);

#endif // WRAPPER_H_
