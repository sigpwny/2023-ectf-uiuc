car:
	cd /sigpwny && cargo build --release --bin car 
	cp /sigpwny/target/thumbv7em-none-eabihf/release/car ${ELF_PATH}
	arm-none-eabi-objcopy -O binary -R .vector_table ${ELF_PATH} ${BIN_PATH}
	python3 build_car_eeprom.py ${SECRETS_DIR} ${EEPROM_PATH} ${CAR_ID}
