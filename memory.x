MEMORY
{
    FLASH (rx)  : ORIGIN = 0x00000000, LENGTH = 0x00040000
    RAM   (rwx) : ORIGIN = 0x20000000, LENGTH = 0x00008000
}

/* Place the .text segment at 0x8000 per eCTF specs */
_stext = ORIGIN(FLASH) + 0x8000;
