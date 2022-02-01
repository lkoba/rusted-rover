SHELL := /bin/bash

APP_NAME := rusted-rover
SERIAL_DEV := /dev/serial/by-id/usb-Silicon_Labs_CP2102_USB_to_UART_Bridge_Controller_0001-if00-port0

.PHONY: clean
clean:
	idf.py fullclean
	(cd components/rust-* && cargo clean)

.PHONY: build
build:
	idf.py build

.PHONY: flash
flash:
	while true; do \
		idf.py -p ${SERIAL_DEV} -b 921600 flash \
		&& break; echo "Retrying ..."; sleep 1; done

.PHONY: ota-server
ota-server:
	cp build/${APP_NAME}.bin ../ota-server/files/ota.bin
	../ota-server/start.sh

.PHONY: erase-flash
erase-flash:
	idf.py -p ${SERIAL_DEV} erase-flash

.PHONY: monitor
monitor:
	idf.py -p ${SERIAL_DEV} monitor

.PHONY: menuconfig
menuconfig:
	idf.py menuconfig
