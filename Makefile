# Try to determine the artifact name. If this does not work replace it with the explicit name.
ARTIFACT := $(shell cargo pkgid |  rev | cut -d "/" -f1  | rev | cut -d "#" -f1)

# Replace this with your ssh configuration for the robot like `robot@192.168.2.3`
TARGET := ev3

all: build

build:
	cargo build --release --target armv5te-unknown-linux-musleabi

deploy:
	scp $(PWD)/target/armv5te-unknown-linux-musleabi/release/$(ARTIFACT) $(TARGET):.

run:
	ssh $(TARGET) brickrun -r ./$(ARTIFACT)
