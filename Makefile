# Try to determine the artifact name. If this does not work replace it with the explicit name.
ARTIFACT := $(shell cargo pkgid |  rev | cut -d "/" -f1  | rev | cut -d "#" -f1)

# Replace this with your ssh configuration for the robot like `robot@192.168.2.3`
TARGET := ev3

all: build

install_rsync:
	wget http://archive.debian.org/debian/pool/main/r/rsync/rsync_3.1.2-1+deb9u2_armel.deb && scp rsync_3.1.2-1+deb9u2_armel.deb $(TARGET):. && ssh -t $(TARGET) sudo dpkg -i rsync_3.1.2-1+deb9u2_armel.deb

cleanup_rsync:
	rm rsync_3.1.2-1+deb9u2_armel.deb && ssh $(TARGET) rm rsync_3.1.2-1+deb9u2_armel.deb

build:
	cargo build --release --target armv5te-unknown-linux-musleabi

deploy:
	rsync --progress -vh $(PWD)/target/armv5te-unknown-linux-musleabi/release/$(ARTIFACT) $(TARGET):.

run:
	ssh $(TARGET) brickrun -r ./$(ARTIFACT)
