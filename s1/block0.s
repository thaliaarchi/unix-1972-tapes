/ Block 0 of s1-bits

/ Tape control registers [https://gunkies.org/wiki/TC11_DECtape_controller]
tcst =	177340				/ #define TCST	0177340	/* Control and status register */
tccm =	177342				/ #define TCCM	0177342	/* Command register */
tcwc =	177344				/ #define TCWC	0177344	/* Word count register */
tcba =	177346				/ #define TCBA	0177346	/* Bus address register */
tcdt =	177350				/ #define TCDT	0177350	/* Data register */
/ Disk control registers [https://gunkies.org/wiki/RF11_disk_controller]
dcs =	177460				/ #define DCS	0177460	/* Disk control status register */
wc =	177462				/ #define WC	0177462	/* Word count register */
cma =	177464				/ #define CMA	0177464	/* Current memory address */
dar =	177466				/ #define DAR	0177466	/* Disk address register */
dae =	177470				/ #define DAE	0177470	/* Disk address extension error register */
dbr =	177472				/ #define DBR	0177472	/* Data buffer register */
ma =	177474				/ #define MA	0177474	/* Maintenance register */
ads =	177476				/ #define ADS	0177476	/* Address of disk segment register */

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

					/ /* TCCM command values */
					/ #define DO	1	/* Give a new function */
					/ #define RNUM	2	/* Function: read block number */
					/ #define TAPE0	00000	/* Select tape unit 0 */
					/ #define FWD	00000	/* Forward direction */
					/ #define REV	04000	/* Reverse direction */

					/ /* Wait until TCCM bit 7 (ready) is set, indicating
					/  * that the current command has completed execution. */
					/ #define wait() while (*(char *)TCCM >= 0)

					/ /* Test whether TCCM bit 15 (error) is set. */
					/ #define error()

init:					/ init(data)
					/ int *data; /* r5 */
					/ {
seekfwd:				/ seekfwd:
	mov	$tcdt,r0
	mov	$tccm,r1
	mov	$3,(r1)			/	*TCCM = DO | RNUM | TAPE0 | FWD;
1:
	tstb	(r1)			/	wait();
	bge	1b
	tst	(r1)	/ error?	/	if (error())
	blt	seekrev			/		goto seekrev;
	cmp	(r5),(r0)		/	if (*data == *TCDT)
	beq	found			/		goto found;
	bgt	seekfwd			/	if (*data > *TCDT)
					/		goto seekfwd;
seekrev:				/ seekrev:
	mov	$4003,(r1)		/	*TCCM = DO | RNUM | TAPE0 | REV;
1:
	tstb	(r1)			/	wait();
	bge	1b
	tst	(r1)			/	if (error())
	blt	seekfwd			/		goto seekfwd;
	mov	(r0),r2
	add	$5,r2
	cmp	(r5),r2			/	if (*data > *TCDT + 5)
	bgt	seekfwd			/		goto seekfwd;
	br	seekrev			/	goto seekrev;
found:					/ found:
	tst	(r5)+			/	data++;
	mov	(r5)+,-(r0)		/	*TCBA = *data++;
	mov	(r5)+,-(r0)		/	*TCWC = *data++;
	mov	(r5)+,-(r0)		/	*TCCM = *data++;
1:
	tstb	(r0)			/	wait();
	bge	1b
	tst	(r0)	/ error?	/	if (error()) {
	bge	2f
	sub	$8.,r5			/		data =- 4;
	br	seekfwd			/		goto seekfwd;
2:					/	}
	mov	$1,(r0)			/	*TCCM = 1;
	rts	r5			/	return;

disk:					/ disk:
	mov	$dbr,r0
	mov	(r5)+,-(r0)		/	*DAE = *data++;
	mov	(r5)+,-(r0)		/	*DAR = *data++;
	mov	(r5)+,-(r0)		/	*CMA = *data++;
	mov	(r5)+,-(r0)		/	*WC = *data++;
	mov	(r5)+,-(r0)		/	*DCS = *data++;
1:
	tstb	(r0)			/	wait();
	bge	1b
	tst	(r0)			/	if (error()) {
	bge	2f
	sub	$10.,r5			/		data =- 5;
	br	disk			/		goto disk;
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
