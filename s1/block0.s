/ Block 0 of s1-bits

/ Tape control registers [https://gunkies.org/wiki/TC11_DECtape_controller]
tcst =	177340				/ #define TCST	((int *)0177340)	/* Control and status register */
tccm =	177342				/ #define TCCM	((int *)0177342)	/* Command register */
tcwc =	177344				/ #define TCWC	((int *)0177344)	/* Word count register */
tcba =	177346				/ #define TCBA	((int *)0177346)	/* Bus address register */
tcdt =	177350				/ #define TCDT	((int *)0177350)	/* Data register */
/ Disk control registers [https://gunkies.org/wiki/RF11_disk_controller]
dcs =	177460				/ #define DCS	((int *)0177460)	/* Disk control status register */
wc =	177462				/ #define WC	((int *)0177462)	/* Word count register */
cma =	177464				/ #define CMA	((int *)0177464)	/* Current memory address */
dar =	177466				/ #define DAR	((int *)0177466)	/* Disk address register */
dae =	177470				/ #define DAE	((int *)0177470)	/* Disk address extension error register */
dbr =	177472				/ #define DBR	((int *)0177472)	/* Data buffer register */
ma =	177474				/ #define MA	((int *)0177474)	/* Maintenance register */
ads =	177476				/ #define ADS	((int *)0177476)	/* Address of disk segment register */

					/ /* TCCM bits */
					/ #define DO		1	/* Give a new function */
					/ #define SAT		(0<<1)	/* Function: stop all transports */
					/ #define RNUM		(1<<1)	/* Function: read block number */
					/ #define RDATA		(2<<1)	/* Function: read data */
					/ #define RALL		(3<<1)	/* Function: read all */
					/ #define SST		(4<<1)	/* Function: stop selected transport */
					/ #define WRTM		(5<<1)	/* Function: write timing and mark trace */
					/ #define WDATA		(6<<1)	/* Function: write data */
					/ #define WALL		(7<<1)	/* Function: write all */
					/ #define XBA16		(1<<4)	/* Extended bus address bit 16 */
					/ #define XBA17		(1<<5)	/* Extended bus address bit 17 */
					/ #define IE		(1<<6)	/* Interrupt enable */
					/ #define READY		(1<<7)	/* Ready */
					/ #define TAPE(n)	(n<<8)	/* Select tape unit n (0-7) */
					/ #define FWD		(0<<11)	/* Forward direction */
					/ #define REV		(1<<11)	/* Reverse direction */
					/ #define DINHB		(1<<12)	/* Delay inhibit */
					/ #define MAINT		(1<<13) /* Used for maintenance functions */
					/ #define ERROR		(1<<15)	/* Error condition */

	mov	$20000,sp		/ #define MEMP	((int *)020000)	/* Base address to copy tape data to */

					/ main()
					/ {
					/	/* Word counts are negated. */
					/
					/	/* Read 8192 words from tape 0 to address MEMP. */
	jsr	r5,tapecmd		/	tapecmd(
	1				/		1,
	20000	/ TCBA			/		MEMP,		/* TCBA */
	-20000	/ TCWC			/		-8192,		/* TCWC */
	5	/ TCCM			/		DO | RALL | TAPE(0) | FWD,	/* TCCM */
					/	);
	jsr	r5,diskcmd		/	diskcmd(
	3	/ DAE			/		3,		/* DAE */
	140000	/ DAR			/		0140000,	/* DAR */
	20000	/ CMA			/		MEMP,		/* CMA */
	-20000	/ WC			/		-8192,		/* WC */
	3	/ DCS			/		3,		/* DCS */
					/	);
	jsr	r5,tapecmd		/	tapecmd(
	41				/		33,
	20000	/ TCBA			/		MEMP,		/* TCBA */
	-20000	/ TCWC			/		-8192,		/* TCWC */
	5	/ TCCM			/		DO | RDATA | TAPE(0) | FWD,	/* TCCM */
					/	);
	jsr	r5,diskcmd		/	diskcmd(
	3	/ DAE			/		3,		/* DAE */
	160000	/ DAR			/		0160000,	/* DAR */
	20000	/ CMA			/		020000,		/* CMA */
	-20000	/ WC			/		-8192,		/* WC */
	3	/ DCS			/		3,		/* DCS */
					/	);
	jsr	r5,diskcmd		/	diskcmd(
	3	/ DAE			/		3,		/* DAE */
	140000	/ DAR			/		0140000,	/* DAR */
	54000	/ CMA			/		054000,		/* CMA */
	-2000	/ WC			/		-1024,		/* WC */
	5	/ DCS			/		5,		/* DCS */
					/	);
	jmp	*$54000			/	goto *054000;
					/ };

					/ /* Wait until TCCM bit 7 (ready) is set, indicating
					/  * that the current command has completed execution. */
					/ #define tcwait() while (*(char *)TCCM >= 0)
					/ /* Test whether TCCM bit 15 (error) is set. */
					/ #define tcerror() (*TCCM < 0)

					/ /* Wait until DCS bit 7 (ready) is set, indicating
					/  * that the current command has completed execution. */
					/ #define dwait() while (*(char *)TCCM >= 0)
					/ /* Test whether DCS bit 15 (error) is set. */
					/ #define derror() (*DCS < 0)

tapecmd:				/ tapecmd(dt, ba, wc, cm)
					/ {
seekfwd:				/ seekfwd:
	mov	$tcdt,r0
	mov	$tccm,r1
	mov	$3,(r1)			/	*TCCM = DO | RNUM | TAPE(0) | FWD;
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
	mov	$4003,(r1)		/	*TCCM = DO | RNUM | TAPE(0) | REV;
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
