/ Block 0 of s1-bits

/ See TC11 DECtape System Manual (DEC-11-HTCB-D) [https://vt100.net/manx/details/1,3529]
/ - Figure 4-2 for TCCM bits

/ Tape control registers
tcst = 177340	/ Control and status register
tccm = 177342	/ Command register
tcwc = 177344	/ Word count register
tcba = 177346	/ Bus address register
tcdt = 177350	/ Data register

	mov	$20000,sp
	jsr	r5,init			/ init(data);

000001; 020000; 160000; 000005		/ int data[] = { ... };
004567; 000204; 000003; 140000
020000; 160000; 000003; 004567
000050; 000041; 020000; 160000
000005; 004567; 000152; 000003
160000; 020000; 160000; 000003
004567; 000134; 000003; 140000
054000; 176000; 000005; 000137
054000

					/ /* Command values */
					/ #define DO	1	/* Give a new function */
					/ #define RNUM	2	/* Function: read block number */
					/ #define TAPE0	00000	/* Select tape unit 0 */
					/ #define FWD	00000	/* Forward direction */
					/ #define REV	04000	/* Reverse direction */

					/ /* Wait until bit 7 (ready) of TCCM is set, indicating
					/  * that the current command has completed execution. */
					/ #define wait() while (*(char *)tccm >= 0)

					/ /* Test whether bit 15 (error) of TCCM is set. */
					/ #define error()

init:					/ init(data)
					/ int *data; /* r5 */
					/ {
					/	register int *tcdt; /* r0 */
					/	register int *tccm; /* r1 */
					/	register x2; /* r2 */
seekfwd:				/ seekfwd:
	mov	$tcdt,r0		/	tcdt = 0177350; /* Tape control data register */
	mov	$tccm,r1		/	tccm = 0177342; /* Tape control command register */
	mov	$3,(r1)			/	*tccm = DO | RNUM | TAPE0 | FWD;
1:
	tstb	(r1)			/	wait();
	bge	1b
	tst	(r1)	/ error?	/	if (error())
	blt	seekrev			/		goto seekrev;
	cmp	(r5),(r0)		/	if (*data == *tcdt)
	beq	found			/		goto found;
	bgt	seekfwd			/	if (*data > *tcdt)
					/		goto seekfwd;
seekrev:				/ seekrev:
	mov	$4003,(r1)		/	*tccm = DO | RNUM | TAPE0 | REV;
1:
	tstb	(r1)			/	wait();
	bge	1b
	tst	(r1)			/	if (error())
	blt	seekfwd			/		goto seekfwd;
	mov	(r0),r2			/	x2 = *tcdt + 5;
	add	$5,r2
	cmp	(r5),r2			/	if (*data > x2)
	bgt	seekfwd			/		goto seekfwd;
	br	seekrev			/	goto seekrev;
found:					/ found:
	tst	(r5)+			/	data++;
	mov	(r5)+,-(r0)		/	*--tcdt = *data++;
	mov	(r5)+,-(r0)		/	*--tcdt = *data++;
	mov	(r5)+,-(r0)		/	*--tcdt = *data++;
1:
	tstb	(r0)			/	wait();
	bge	1b
	tst	(r0)	/ error?	/	if (error()) {
	bge	2f
	sub	$10,r5			/		data =- 4;
	br	seekfwd			/		goto seekfwd;
2:					/	}
	mov	$1,(r0)			/	*tcdt = 1;
	rts	r5			/	return;

todo:					/ todo:
	mov	$177472,r0		/	tcdt = 0177472; /* TODO */
	mov	(r5)+,-(r0)		/	*--tcdt = *data++;
	mov	(r5)+,-(r0)		/	*--tcdt = *data++;
	mov	(r5)+,-(r0)		/	*--tcdt = *data++;
	mov	(r5)+,-(r0)		/	*--tcdt = *data++;
	mov	(r5)+,-(r0)		/	*--tcdt = *data++;
1:
	tstb	(r0)			/	wait();
	bge	1b
	tst	(r0)			/	if (error()) {
	bge	2f
	sub	$12,r5			/		data =- 5;
	br	todo			/		goto todo;
2:					/	}
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
