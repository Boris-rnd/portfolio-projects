ENTRY(_start)

SECTIONS {
    . = 0x7c00+512;

    .start : {
        *(.start)
    }
    .rust :
    {
        *(.text .text.*)
        *(.rodata .rodata.*)
    }

    /DISCARD/ : {
        *(.comment .comment.*)
    }

   /DISCARD/ : 
   {
    *(.comment .comment.*)
   }

   /*
    * ensures that our signature ends on a 512 byte boundry which is the size of one sector off of disk
    * This is important as we can only read 512 bytes at a time and we want to ensure that our signature is at 
    * the end of this boundry so we don't partially miss it
    * */
   . += 4; 
   . = ALIGN(512) - 4;
   .end : 
   {
       LONG(0xdeadbeaf)
   }

}

/*
 * NOTE TO SELF:
 * (My sneaking suspicion is that it takes a very long time for it to be aligned on a 512 byte boundry if you are
 * adding 512 bytes to the pointer)
 * . += ALIGN(512); 
 * . = . - 4; 
 * 
 * Makes the binary WAY too big. I'm not sure why, TODO look into this!
 *
 * For now do this instead:
 * 
 * . += 4;
 * . = ALIGN(512) - 4; 
 * */
