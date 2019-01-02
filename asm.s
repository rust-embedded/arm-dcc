  .section .text.__dcc_write
  .global __dcc_write
__dcc_write:
1:  mrc     p14, 0, r1, c0, c1, 0
    tst     r1, #536870912      /* 0x20000000 */
    bne     1b
    mcr     p14, 0, r0, c0, c5, 0
    bx      lr
