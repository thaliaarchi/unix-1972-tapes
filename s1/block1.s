/ Block 1 of s1-bits

/ It was written by /usr/sys/maki.s in s2 and was populated from the contents of
/ buf when that program was run.

/ Documented in V1 and V2 bproc(7).
/ This version appears to be between settings in the V1 and V2 manuals. It has
/ the following settings:
/   *	???	V2 (was 173700 or 73700 in V1)
/   *	0	both
/   *	1	both
/   *	2	V1 only
/   *	10	both
/   *	20	V2 only
/   *	40	V2 only
/   *	57500	V1 (changed to 77500 in V2)

/ This program is loaded at address 054000.
base = 54000

sr =	177570				/ #define SR	((int *)0177570)	/* Switch register */
					/ /* Tape control registers */
tccm =	177342				/ #define TCCM	((int *)0177342)	/* Command register */
					/ /* Disk control registers */
					/ #define DCS	((int *)0177460)	/* Disk control status register */
					/ #define WC	((int *)0177462)	/* Word count register */
					/ #define CMA	((int *)0177464)	/* Current memory address */
					/ #define DAR	((int *)0177466)	/* Disk address register */
					/ #define DAE	((int *)0177470)	/* Disk address extension error register */
dbr =	177472				/ #define DBR	((int *)0177472)	/* Data buffer register */

block1:					/ block1()
					/ {
					/	register int *t; /* r1 */
	mov	pc,sp	/ pc == 54000
	mov	$base+tab,r1		/	/* Search the table for a key matching SR */
1:					/	for (t = tab; t != tab + 7; t++) {
	cmp	*$sr,(r1)+		/		if (*SR == t->srval)
	beq	2f			/			return(t->f());
	tst	(r1)+
	cmp	r1,$base+L1
	bne	1b			/	}
	br	L1			/	goto L1; /* Warm Unix */
2:
	jmp	*0(r1)			/
					/	struct entry {
					/		int srval;
					/		int (*f)();
					/	};
tab:					/	struct entry tab[] = {
	000000; base+L5			/		{000000, L5},	/* Load standard Unix binary paper tape and transfer to it */
	057500; base+L6			/		{057500, L6},	/* Load standard DEC absolute and binary loaders and transfer to it */
	000010; base+L8			/		{000010, L8},	/* Dump memory onto a drive then halt */
	000020; base+L12		/		{000020, L12},	/* Read 256 words from disk into memory and transfer to it */
	000040; base+L9			/		{000040, L9},	/* Dump memory onto a drive then load Warm Unix */
	000001; base+L2			/		{000001, L2},	/* Cold Unix */
	000002; base+L3			/		{000002, L3},	/* Read the unassigned 3K program and transfer to it */
					/	};
L1:					/ L1:	/* Warm Unix */
					/	/* Read 6144 words from disk 0 track 120 word 1024 to address 0.
					/	 * This is at word 120*2048+1024 from the disk start,
					/	 * which was copied from words 1280-7424 in the tape. */
	jsr	r0,diskcmd		/	diskcmd(
	3	/ DAE			/		03|DISK(0),	/* DAE: track address (bits 5-6) and disk */
	142000	/ DAR			/		02000|(030<<11),	/* DAR: word address and track address (bits 0-4)
	0	/ CMA			/		0,		/* CMA: memory address */
	-14000	/ WC			/		-6144,		/* WC: transfer 6144 words */
	5	/ DCS			/		GO|D_READ,	/* DCS: command */
					/	);
L2:					/ L2:
					/	/* Read 7680 words from disk 0 track 123 word 1024 to address 0.
					/	 * This is word 123*2048+1024 from the disk start. */
	jsr	r0,diskcmd		/	diskcmd(
	3	/ DAE			/		03|DISK(0),	/* DAE: track address (bits 5-6) and disk */
	156000	/ DAR			/		02000|(033<<11),	/* DAR: word address and track address (bits 0-4)
	0	/ CMA			/		0,		/* CMA: memory address */
	-17000	/ WC			/		-7680,		/* WC: transfer 7680 words */
	5	/ DCS			/		GO|D_READ,	/* DCS: command */
					/	);
L3:					/ L3:
					/	/* Read 1536 words from disk 0 track 126 word 1024 to address 0.
					/	 * This is word 126*2048+1024 from the disk start. */
	jsr	r0,diskcmd		/	diskcmd(
	3	/ DAE			/		03|DISK(0),	/* DAE: track address (bits 5-6) and disk */
	172000	/ DAR			/		02000|(036<<11),	/* DAR: word address and track address (bits 0-4)
	0	/ CMA			/		0,		/* CMA: memory address */
	-3000	/ WC			/		-1536,		/* WC: transfer 1536 words */
	5	/ DCS			/		GO|D_READ,	/* DCS: command */
					/	);

					/ /* Sends the given command to the disk controller. */
diskcmd:				/ diskcmd(dae, dar, cma, wc, dcs)
					/ {
					/	int s;
	mov	$dbr,r1
	mov	(r0)+,-(r1)		/	*DAE = dae;
	mov	(r0)+,-(r1)		/	*DAR = dar;
	mov	(r0)+,-(r1)		/	*CMA = cma;
	mov	(r0)+,-(r1)		/	*WC = wc;
	mov	(r0)+,-(r1)		/	*DCS = dcs;
1:					/	do {
	mov	(r1),r0			/		s = *DCS;
	blt	block1			/		if (s < 0)
					/			goto block1;
	tstb	r0			/	/* Until the DCS ready bit is set */
	bge	1b			/	} while((char)s >= 0);
	000167; 124206	/ jmp 125400	/	goto TODO;
					/	/* Apparently does not pop sp */
					/ }

L5:
	jsr	r0,L7
	base+410
L6:
	jsr	r0,L7
	base+550
L7:
	mov	(r0),r0
	mov	$57500,r1
1:
	mov	(r0)+,(r1)+
	cmp	r1,$60000
	bne	1b
	000167; 003250	/ jmp 4500
L8:
	jsr	pc,L10
	000000
	jmp	block1
L9:
	jsr	pc,L10
	jmp	L1
L10:
	mov	$tccm,r5
L11:
	mov	$7403,(r5)
1:
	tstb	(r5)
	bge	1b
	tst	(r5)
	bge	L11
	tst	177776(r5)
	bge	L11
	mov	$177350,r1
	mov	$3403,(r5)
1:
	tstb	(r5)
	bge	1b
	tst	(r5)
	blt	L11
	tst	(r1)
	bne	L11
	clr	-(r1)
	mov	$150000,-(r1)
	mov	$3415,-(r1)
1:
	tstb	(r5)
	bge	1b
	tst	(r5)
	blt	L11
	mov	$7403,(r5)
	rts	pc
L12:
	mov	$177414,r1
	clr	-(r1)
	clr	-(r1)
	mov	$177400,-(r1)
	mov	$5,-(r1)
1:
	tstb	(r1)
	bge	1b
	tst	(r1)
	blt	L12
	jmp	*$0
L13:
	mov	pc,sp
	clr	r5
	clr	r1
L14:
	jsr	pc,L19
	tst	r0
	beq	L14
	mov	r0,r2
	bge	L15
1:
	clrb	(r5)+
	inc	r2
	bne	1b
	jsr	pc,L17
	br	L14
L15:
	dec	r2
	bne	L16
	jsr	pc,L17
	000167; 120216	/ jmp 121700
L16:
	jsr	pc,L19
	movb	r0,(r5)+
	dec	r2
	bne	L16
	jsr	pc,L17
	br	L14
L17:
	jsr	pc,L19
	tstb	r1
	bne	1f
	rts	pc
1:
	000000 / halt

	br	L13
L19:
	005267; 117324	/ inc 121050
L20:
	005767; 117320	/ tst 121050
	blt	L19
	105767; 117312	/ tstb 121050
	bge	L20
	116700; 117306	/ movb 121052,r0
	add	r0,r1
	rts	pc

	mov	pc,sp
	cmp	-(sp),-(sp)
	mov	pc,r5
	add	$114,r5
	clr	r1
L21:
	mov	*$177570,(sp)
	ror	(sp)
	bcs	1f
	clr	(sp)
	br	L22
1:
	clc
	rol	(sp)
	bne	L22
	mov	r1,(sp)
L22:
	clr	r0
	jsr	pc,(r5)
	decb	r3
	bne	L22
	jsr	pc,(r5)
	jsr	pc,L25
	mov	r4,r2
	sub	$4,r2
	cmp	$2,r2
	beq	L26
	jsr	pc,L25
	add	(sp),r4
	mov	r4,r1
L23:
	jsr	pc,(r5)
	bge	1f
	tstb	r0
	beq	L22
L24:
	000000	/ halt
	br	L22
1:
	movb	r3,(r1)+
	br	L23

	016703; 000150	/ mov 2046,r3
	incb	(r3)
1:
	tstb	(r3)
	bpl	1b
	movb	2(r3),r3
	add	r3,r0
	bic	$177400,r3
	dec	r2
	rts	pc
L25:
	mov	(sp)+,L27
	jsr	pc,(r5)
	mov	r3,r4
	jsr	pc,(r5)
	swab	r3
	bis	r3,r4
	mov	L27,pc
L26:
	jsr	pc,L25
	jsr	pc,(r5)
	tstb	r0
	bne	L24
	asr	r4
	bcc	1f
	000000	/ halt
	br	L21
1:
	asl	r4
	jmp	(r4)
L27:
	000000	/ halt
	000000	/ halt
	000000	/ halt
