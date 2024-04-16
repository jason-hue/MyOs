//
// Created by knifefire on 24-4-16.
//

#include "printf.h"
#include <stdarg.h>
#include "types.h"
#include "libc.h"
static char digits[] = "0123456789abcdef";

static void printint(int xx, int base, int sign)
{
    char buf[16];
    int i;
    uint x;

    if (sign && (sign = xx < 0))
        x = -xx;
    else
        x = xx;

    i = 0;
    do {
        buf[i++] = digits[x % base];
    } while ((x /= base) != 0);

    if (sign)
        buf[i++] = '-';

    while (--i >= 0)
        write(&buf[i]);
}

static void printptr(uint64 x)
{
    int i;
    write((char *) '0');
    write((char *) 'x');
    for (i = 0; i < (sizeof(uint64) * 2); i++, x <<= 4)
        write(&digits[x >> (sizeof(uint64) * 8 - 4)]);
}

void printf(char *fmt, ...)
{
    va_list ap;
    int i, c;
    char *s;

    if (fmt == 0)
        //panic("null fmt");

    va_start(ap, fmt);
    for (i = 0; (c = fmt[i] & 0xff) != 0; i++) {
        if (c != '%') {
            write((char *) c);
            continue;
        }
        c = fmt[++i] & 0xff;
        if (c == 0)
            break;
        switch (c) {
            case 'd':
                printint(va_arg(ap, int), 10, 1);
                break;
            case 'x':
                printint(va_arg(ap, int), 16, 1);
                break;
            case 'p':
                printptr(va_arg(ap, uint64));
                break;
            case 's':
                if ((s = va_arg(ap, char *)) == 0)
                    s = "(null)";
                for (; *s; s++)
                    write(s);
                break;
            case '%':
                write((char *) '%');
                break;
            default:
                write((char *) '%');
                write((char *) c);
                break;
        }
    }
}