/ Block 1 of s1-bits

sr = 177570	/ Switch register

					/	register int *p; /* r1 */
	mov	pc,sp			/	?
	mov	$54032,r1		/	p = 054032
1:					/	for (;;) {
	cmp	*$sr,(r1)+		/		if (*SR == *p++)
	beq	2f			/			break;
	tst	(r1)+
	cmp	r1,$54066		/		if (++p != 054066)
	bne	1b			/			continue;
	br	3f			/		goto L3;
2:					/	}
	jmp	*0(r1)			/	goto p[0];

000000; 054172; 057500; 054200
000010; 054230; 000020; 054354
000040; 054242; 000001; 054104
000002; 054122

3:					/ L3:
