/ Block 0 of s1-bits

/ This reads data from the tape and reads from and writes to a disk by
/ manipulating device registers for the RC11 DECtape controller and RF11 DECdisk
/ controller.
/
/ References:
/   *	TC11 DECtape system manual, Chapter 4: Programming Information,
/	https://bitsavers.org/pdf/dec/dectape/tc11/TC11_Mantenance_Manual.pdf
/   *	RF11/RS11 DECdisk system manual, Chapter 3: Programming,
/	https://bitsavers.org/pdf/dec/unibus/DEC-11-HRFB-D__RF11_RS11_DECdisk_System_Manual__Aug71.pdf
/   *	https://gunkies.org/wiki/TC11_DECtape_controller
/   *	https://gunkies.org/wiki/RF11_disk_controller

/ Tape control registers
tcst =	177340				/ #define TCST	((int *)0177340)	/* Control and status register */
tccm =	177342				/ #define TCCM	((int *)0177342)	/* Command register */
tcwc =	177344				/ #define TCWC	((int *)0177344)	/* Word count register */
tcba =	177346				/ #define TCBA	((int *)0177346)	/* Bus address register */
tcdt =	177350				/ #define TCDT	((int *)0177350)	/* Data register */
/ Disk control registers
dcs =	177460				/ #define DCS	((int *)0177460)	/* Disk control status register */
wc =	177462				/ #define WC	((int *)0177462)	/* Word count register */
cma =	177464				/ #define CMA	((int *)0177464)	/* Current memory address */
dar =	177466				/ #define DAR	((int *)0177466)	/* Disk address register */
dae =	177470				/ #define DAE	((int *)0177470)	/* Disk address extension error register */
dbr =	177472				/ #define DBR	((int *)0177472)	/* Data buffer register */
ma =	177474				/ #define MA	((int *)0177474)	/* Maintenance register */
ads =	177476				/ #define ADS	((int *)0177476)	/* Address of disk segment register */

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
					/ #define D_READ	(1<<1)	/* Function: read */
					/ #define D_WRITE	(2<<1)	/* Function: write */
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

	mov	$20000,sp		/ #define MEMP	((int *)020000)	/* Base address to copy tape data to */

					/ main()
					/ {
					/	/* Read 8192 words from tape 0 to address MEMP */
	jsr	r5,tapecmd		/	tapecmd(
	1				/		1,
	20000	/ TCBA			/		MEMP,		/* TCBA: memory address */
	-20000	/ TCWC			/		-8192,		/* TCWC: transfer 8192 words */
	5	/ TCCM			/		GO|T_RALL|TAPE(0)|T_FWD,	/* TCCM */
					/	);
					/	/* Read 8192 words from disk 0 track 120 to address MEMP */
	jsr	r5,diskcmd		/	diskcmd(
	3	/ DAE			/		03|DISK(0),	/* DAE: track address (bits 5-6) and disk */
	140000	/ DAR			/		030<<11,	/* DAR: track address (bits 0-4) */
	20000	/ CMA			/		MEMP,		/* CMA: memory address */
	-20000	/ WC			/		-8192,		/* WC: transfer 8192 words */
	3	/ DCS			/		GO|D_READ,	/* DCS */
					/	);
	jsr	r5,tapecmd		/	tapecmd(
	41				/		33,
	20000	/ TCBA			/		MEMP,		/* TCBA: memory address */
	-20000	/ TCWC			/		-8192,		/* TCWC: transfer 8192 words */
	5	/ TCCM			/		GO|T_RDATA|TAPE(0)|T_FWD,	/* TCCM */
					/	);
					/	/* Read 8192 words from disk 0 track 124 to address MEMP */
	jsr	r5,diskcmd		/	diskcmd(
	3	/ DAE			/		03|DISK(0),	/* DAE: track address (bits 5-6) and disk */
	160000	/ DAR			/		034<<11,	/* DAR: track address (bits 0-4) */
	20000	/ CMA			/		MEMP,		/* CMA: memory address */
	-20000	/ WC			/		-8192,		/* WC: transfer 8192 words */
	3	/ DCS			/		GO|D_READ,	/* DCS */
					/	);
					/	/* Write 8192 words from address 054000 to disk 0 track 120 */
	jsr	r5,diskcmd		/	diskcmd(
	3	/ DAE			/		03|DISK(0),	/* DAE: track address (bits 5-6) and disk */
	140000	/ DAR			/		030<<11,	/* DAR: track address (bits 0-4) */
	54000	/ CMA			/		054000,		/* CMA: memory address */
	-2000	/ WC			/		-1024,		/* WC: transfer 8192 words */
	5	/ DCS			/		GO|D_WRITE,	/* DCS */
					/	);
	jmp	*$54000			/	goto *054000;
					/ };

tapecmd:				/ tapecmd(dt, ba, wc, cm)
					/ {
seekfwd:				/ seekfwd:
	mov	$tcdt,r0
	mov	$tccm,r1
	mov	$3,(r1)			/	*TCCM = GO | T_RNUM | TAPE(0) | T_FWD;
1:
	tstb	(r1)			/	tcwait();
	bge	1b
	tst	(r1)	/ error?	/	if (tcerror())
	blt	seekrev			/		goto seekrev;
	cmp	(r5),(r0)		/	if (dt == *TCDT)
	beq	found			/		goto found;
	bgt	seekfwd			/	if (dt > *TCDT)
					/		goto seekfwd;
seekrev:				/ seekrev:
	mov	$4003,(r1)		/	*TCCM = GO | T_RNUM | TAPE(0) | T_REV;
1:
	tstb	(r1)			/	tcwait();
	bge	1b
	tst	(r1)			/	if (tcerror())
	blt	seekfwd			/		goto seekfwd;
	mov	(r0),r2
	add	$5,r2
	cmp	(r5),r2			/	if (dt > *TCDT + 5)
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
	mov	$1,(r0)			/	*TCCM = 1;
	rts	r5
					/ }

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
