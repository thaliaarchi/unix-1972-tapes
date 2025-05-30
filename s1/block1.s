/ Block 1 of s1-bits

/ This program is loaded at address 054000.
base = 54000				/ #define BASE	((char *)54000)

sr =	177570				/ #define SR	((int *)0177570)	/* Switch register */
					/ /* Tape control registers */
tccm =	177342				/ #define TCCM	((int *)0177342)	/* Command register */
					/ /* Disk control registers */
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
	br	L1			/	goto L1;
2:
	jmp	*0(r1)			/
					/	struct entry {
					/		int srval;
					/		int (*f)();
					/	};
tab:					/	struct entry tab[] = {
	000000; base+L5			/		{000000, BASE+L5},
	057500; base+L6			/		{057500, BASE+L6},
	000010; base+L8			/		{000010, BASE+L8},
	000020; base+L12		/		{000020, BASE+L12},
	000040; base+L9			/		{000040, BASE+L9},
	000001; base+L2			/		{000001, BASE+L2},
	000002; base+L3			/		{000002, BASE+L3},
					/	};
L1:
	jsr	r0,L4
	000003; 142000; 000000; 164000; 000005
L2:
	jsr	r0,L4
	000003; 156000; 000000; 161000; 000005
L3:
	jsr	r0,L4
	000003; 172000; 000000; 175000; 000005
L4:
	mov	$dbr,r1
	mov	(r0)+,-(r1)
	mov	(r0)+,-(r1)
	mov	(r0)+,-(r1)
	mov	(r0)+,-(r1)
	mov	(r0)+,-(r1)
1:
	mov	(r1),r0
	blt	block1
	tstb	r0
	bge	1b
	000167; 124206	/ jmp 125400
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
