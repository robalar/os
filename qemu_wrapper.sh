KERNEL="$1"

is_test=false
if [[ "$KERNEL" == *"deps"* ]]; then
    is_test=true
fi

echo "Is test: $is_test"

qemu-system-i386 -kernel "$KERNEL" \
	-device isa-debug-exit,iobase=0xf4,iosize=0x04 # Allow exiting QEMU from guest
qemu_exit_code=$?

if [[ "$is_test" == true ]]; then
    if [[ "$qemu_exit_code" == 33 ]]; then
       exit 0
    else 
	exit 1
    fi
else
    exit $qemu_exit_code
fi
