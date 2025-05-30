/ Block 1 of s1-bits

/ This program is loaded at address 054000.
base = 54000				/ #define BASE	((char *)54000)

sr =	177570				/ #define SR	((int *)0177570)	/* Switch register */

block1:					/ block1()
					/ {
					/	register int *t; /* r1 */
	mov	pc,sp	/ pc == 54000
	mov	$base+tab,r1		/	/* Search the table for a key matching SR */
1:					/	for (t = tab; t != tab + 7; t++) {
	cmp	*$sr,(r1)+		/		if (*SR == t->srval)
	beq	2f			/			return(t->f());
	tst	(r1)+
	cmp	r1,$54066
	bne	1b			/	}
	br	3f
2:
	jmp	*0(r1)			/
					/	struct entry {
					/		int srval;
					/		int (*f)();
					/	};
tab:					/	struct entry tab[] = {
	000000; base+172		/		{000000, BASE+172},
	057500; base+200		/		{057500, BASE+200},
	000010; base+230		/		{000010, BASE+230},
	000020; base+354		/		{000020, BASE+354},
	000040; base+242		/		{000040, BASE+242},
	000001; base+104		/		{000001, BASE+104},
	000002; base+122		/		{000002, BASE+122},
					/	};
3:
