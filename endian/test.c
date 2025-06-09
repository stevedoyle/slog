#include <stdio.h>
#include <stdint.h>

struct Foo
{
    uint8_t x;
    uint8_t y;
    uint16_t z;
};

int main()
{
    uint8_t* tmp = 0;
    struct Foo f;
    f.x = 0x12;
    f.y = 0x34;
    f.z = 0x5678;

    printf("As a 32 bit value: %8x\n", *(uint32_t*)(&f));
    tmp = &f;
    for(int i=0; i<4; i++) {
        printf("tmp+%d: %02x\n", i, *(tmp+i));
    }

    return 0;
}
