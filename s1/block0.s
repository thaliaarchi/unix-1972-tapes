/ Block 0 of s1-bits

/ This reads data from the tape, writes it to disk, then executes it.
/
/ This program is from bytes 0-290 of the tape. It copies 32768 bytes from the
/ tape at bytes 512-33280 to the disk at bytes 491520-524288. Afterwards, bytes
/ 16896-31232 followed by bytes 512-16896 from the tape are loaded into memory
/ at address 020000. It then executes the code at address 054000, which is from
/ byte offset 512 in the disk.
/
/ It manipulates device registers for the RC11 DECtape controller and RF11
/ DECdisk controller. References:
/   *	TC11 DECtape system manual, Chapter 4 Programming Information and Table 3-3,
/	https://bitsavers.org/pdf/dec/dectape/tc11/TC11_Mantenance_Manual.pdf
/   *	RF11/RS11 DECdisk system manual, Chapter 3 Programming and Section 1.3.3 Logic Format,
/	https://bitsavers.org/pdf/dec/unibus/DEC-11-HRFB-D__RF11_RS11_DECdisk_System_Manual__Aug71.pdf
/   *	https://gunkies.org/wiki/TC11_DECtape_controller
/   *	https://gunkies.org/wiki/RF11_disk_controller

					/ /* Tape control registers */
					/ #define TCST	((int *)0177340)	/* Control and status register */
tccm =	177342				/ #define TCCM	((int *)0177342)	/* Command register */
					/ #define TCWC	((int *)0177344)	/* Word count register */
					/ #define TCBA	((int *)0177346)	/* Bus address register */
tcdt =	177350				/ #define TCDT	((int *)0177350)	/* Data register */

					/ /* Disk control registers */
					/ #define DCS	((int *)0177460)	/* Disk control status register */
					/ #define WC	((int *)0177462)	/* Word count register */
					/ #define CMA	((int *)0177464)	/* Current memory address */
					/ #define DAR	((int *)0177466)	/* Disk address register */
					/ #define DAE	((int *)0177470)	/* Disk address extension error register */
dbr =	177472				/ #define DBR	((int *)0177472)	/* Data buffer register */
					/ #define MA	((int *)0177474)	/* Maintenance register */
					/ #define ADS	((int *)0177476)	/* Address of disk segment register */

					/ /* Common TCCM and DCS bits */
					/ #define GO		1	/* Start executing a new function */
					/ #define READY		(1<<7)	/* Ready */
					/ #define ERROR		(1<<15)	/* Error condition */

					/ /* TCCM bits */
					/ #define T_SAT		(0<<1)	/* Function: stop all transports */
					/ #define T_RNUM	(1<<1)	/* Function: read block number */
					/ #define T_RDATA	(2<<1)	/* Function: read data */
					/ #define T_RALL	(3<<1)	/* Function: read all */
					/ #define T_SST		(4<<1)	/* Function: stop selected transport */
					/ #define T_WRTM	(5<<1)	/* Function: write timing and mark trace */
					/ #define T_WDATA	(6<<1)	/* Function: write data */
					/ #define T_WALL	(7<<1)	/* Function: write all */
					/ #define TAPE(n)	(n<<8)	/* Select tape unit n (0-7) */
					/ #define T_FWD		(0<<11)	/* Forward direction */
					/ #define T_REV		(1<<11)	/* Reverse direction */

					/ /* DCS bits */
					/ #define D_NOP		(0<<1)	/* Function: nop */
					/ #define D_WRITE	(1<<1)	/* Function: write */
					/ #define D_READ	(2<<1)	/* Function: read */
					/ #define D_WRTCK	(3<<1)	/* Function: write check */

					/ /* DAE bits */
					/ #define DISK(n)	(n<<2)	/* Select disk n (0-7) */

					/ /* Wait until the TCCM ready bit is set, indicating
					/  * that the current command has completed execution. */
					/ #define tcwait() while (*(char *)TCCM >= 0)
					/ /* Test whether the TCCM error bit is set. */
					/ #define tcerror() (*TCCM < 0)

					/ /* Wait until the DCS ready bit is set, indicating
					/  * that the current command has completed execution. */
					/ #define dwait() while (*(char *)TCCM >= 0)
					/ /* Test whether the DCS error bit is set. */
					/ #define derror() (*DCS < 0)

	mov	$20000,sp		/ int buf[8192+7168];	/* Buffer at address 020000 */

					/ main()
					/ {
					/	/* Read 8192 words from tape 0 block 1 to buf.
					/	 * Block 1 is at word 1*256. */
	jsr	r5,tapecmd		/	tapecmd(
	1	/ Block number		/		1,		/* Block number 1 */
	20000	/ TCBA			/		buf,		/* TCBA: memory address */
	-20000	/ TCWC			/		-8192,		/* TCWC: transfer 8192 words */
	5	/ TCCM			/		GO|T_RALL|TAPE(0)|T_FWD,	/* TCCM: command */
					/	);
					/	/* Write 8192 words from buf to disk 0 track 120.
					/	 * Track 120 is at word 120*2048. */
	jsr	r5,diskcmd		/	diskcmd(
	3	/ DAE			/		03|DISK(0),	/* DAE: track address (bits 5-6) and disk */
	140000	/ DAR			/		030<<11,	/* DAR: track address (bits 0-4) */
	20000	/ CMA			/		buf,		/* CMA: memory address */
	-20000	/ WC			/		-8192,		/* WC: transfer 8192 words */
	3	/ DCS			/		GO|D_WRITE,	/* DCS: command */
					/	);
					/	/* Read 8192 words from tape 0 block 33 to buf.
					/	 * Block 33 is at word 33*256, or 8192 words after block 1. */
	jsr	r5,tapecmd		/	tapecmd(
	41	/ Block number		/		33,		/* Block number 33 */
	20000	/ TCBA			/		buf,		/* TCBA: memory address */
	-20000	/ TCWC			/		-8192,		/* TCWC: transfer 8192 words */
	5	/ TCCM			/		GO|T_RALL|TAPE(0)|T_FWD,	/* TCCM: command */
					/	);
					/	/* Write 8192 words from buf to disk 0 track 124.
					/	 * Track 124 is at word 124*2048, or 8192 words after track 120. */
	jsr	r5,diskcmd		/	diskcmd(
	3	/ DAE			/		03|DISK(0),	/* DAE: track address (bits 5-6) and disk */
	160000	/ DAR			/		034<<11,	/* DAR: track address (bits 0-4) */
	20000	/ CMA			/		buf,		/* CMA: memory address */
	-20000	/ WC			/		-8192,		/* WC: transfer 8192 words */
	3	/ DCS			/		GO|D_WRITE,	/* DCS: command */
					/	);
					/	/* Read 8192 words from disk 0 track 120 to address &buf[7168]. */
	jsr	r5,diskcmd		/	diskcmd(
	3	/ DAE			/		03|DISK(0),	/* DAE: track address (bits 5-6) and disk */
	140000	/ DAR			/		030<<11,	/* DAR: track address (bits 0-4) */
	54000	/ CMA			/		&buf[7168],	/* CMA: memory address */
	-2000	/ WC			/		-1024,		/* WC: transfer 8192 words */
	5	/ DCS			/		GO|D_READ,	/* DCS: command */
					/	);
	jmp	*$54000			/	((int (*)()) &buf[7168])();
					/ };

					/ /* Seeks to the given block in tape 0, then writes
					/  * ba, wc, and cm to TCBA, TCWC, and TCCM, respectively.
tapecmd:				/ tapecmd(block, ba, wc, cm)
					/ {
					/	int b;
seekfwd:				/ seekfwd:
	mov	$tcdt,r0
	mov	$tccm,r1
					/	/* Seek forward in tape 0 to the next block
					/	 * and read its block number into TCDT */
	mov	$3,(r1)			/	*TCCM = GO|T_RNUM|TAPE(0)|T_FWD;
1:
	tstb	(r1)			/	tcwait();
	bge	1b
	tst	(r1)	/ error?	/	if (tcerror())
	blt	seekrev			/		goto seekrev;
					/	b = *TCDT;
	cmp	(r5),(r0)		/	if (b == block)
	beq	found			/		goto found;
	bgt	seekfwd			/	if (b < block)
					/		goto seekfwd;
seekrev:				/ seekrev:
					/	/* Seek backward in tape 0 to the next block
					/	 * and read its block number into TCDT */
	mov	$4003,(r1)		/	*TCCM = GO|T_RNUM|TAPE(0)|T_REV;
1:
	tstb	(r1)			/	tcwait();
	bge	1b
	tst	(r1)			/	if (tcerror())
	blt	seekfwd			/		goto seekfwd;
	mov	(r0),r2
	add	$5,r2			/	/* Let it over-seek, before switching back to forward */
	cmp	(r5),r2			/	if (*TCDT + 5 < block)
	bgt	seekfwd			/		goto seekfwd;
	br	seekrev			/	goto seekrev;
found:					/ found:
	tst	(r5)+
	mov	(r5)+,-(r0)		/	*TCBA = ba;
	mov	(r5)+,-(r0)		/	*TCWC = wc;
	mov	(r5)+,-(r0)		/	*TCCM = cm;
1:
	tstb	(r0)			/	tcwait();
	bge	1b
	tst	(r0)	/ error?	/	if (tcerror())
	bge	2f
	sub	$8.,r5
	br	seekfwd			/		goto seekfwd;
2:
					/	/* Stop tape motion for tape 0 */
	mov	$1,(r0)			/	*TCCM = GO|T_SAT|TAPE(0);
	rts	r5
					/ }

					/ /* Sends the given command to the disk controller. */
diskcmd:				/ diskcmd(dae, dar, cma, wc, dcs)
					/ {
					/ retry:
	mov	$dbr,r0
	mov	(r5)+,-(r0)		/	*DAE = dae;
	mov	(r5)+,-(r0)		/	*DAR = dar;
	mov	(r5)+,-(r0)		/	*CMA = cma;
	mov	(r5)+,-(r0)		/	*WC = wc;
	mov	(r5)+,-(r0)		/	*DCS = dcs;
1:
	tstb	(r0)			/	dwait();
	bge	1b
	tst	(r0)			/	if (derror())
	bge	2f
	sub	$10.,r5
	br	diskcmd			/		goto retry;
2:
	rts	r5
					/ }

</dev/tap7\0>
</dev/rf0\0>
</etc/init\0>
</etc/getty\0>
</bin/chmod\0>
</bin/date\0>
</bin/login\0>
</bin/mkdir\0>
</bin/sh\0>
</bin/tap\0>
</bin/ls\0>
