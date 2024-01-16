#/****************************************************
# Programmierpraktikum WISE 24 - Compilation targets
#****************************************************/

INCDIR    = .
DRIVERDIR = ./driver
# Add your header files
HEADERS   = $(INCDIR)/server.h

# Add your source files
DRIVER_UNIT = $(DRIVERDIR)/unittests.c
DRIVER_BENCHMARK = $(DRIVERDIR)/speed_test.c

INCLUDE   = -I. -I$(INCDIR)
DSYS      = BSD42
COMPILER = gcc
DVERSION  =
CFLAGS = -O3 -Wall -mmmx -fPIC -combine
LDFLAGS= -lpthread -W -Wno-sign-compare
MINFLAGS = -O3 -mmmx -fPIC -lpthread -combine
MAXFLAGS_UBUNTU = -O3 -fPIC -lpthread
MAXFLAGS = -O3 -fPIC -lpthread -combine

IMPLEMENTATION ?= ppds # speedLinux or ppds (default: ppds)

rust:
	cargo build --release
	mkdir -p ./out
	cp ./target/release/libppds.so ./out/libppds.so

harness_test: rust $(HEADERS)
	$(COMPILER) $(MAXFLAGS_UBUNTU) $(INCLUDE) $(DRIVER_BENCHMARK) -o ./out/speed_test -Lbin -Lout -l$(IMPLEMENTATION) $(LDFLAGS)

unit_test: rust $(HEADERS)
	$(COMPILER) $(MAXFLAGS_UBUNTU) $(INCLUDE) $(DRIVER_UNIT) -o ./out/unit_test -Lbin -Lout -l$(IMPLEMENTATION) $(LDFLAGS)

all: unit_test harness_test

test_unit: unit_test
	LD_LIBRARY_PATH=out:bin ./out/unit_test

test_speed: harness_test
	LD_LIBRARY_PATH=out:bin ./out/speed_test

test: test_unit test_speed

clean:
	- rm -f *.results
	- rm -rf out target
