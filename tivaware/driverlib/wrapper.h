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
extern bool read_sw_1(void);
extern void get_temp_samples(uint32_t* samples);
extern void sleep_us(uint32_t us);
extern void start_delay_timer_us(uint32_t us);
extern void wait_delay_timer(void);
extern uint32_t get_remaining_us_delay_timer(void);
extern uint64_t get_tick_timer(void);

#endif // WRAPPER_H_
