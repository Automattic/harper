> <!--
# Unlintable
>            source: https://en.wikipedia.org/w/index.php?title=Part-of-speech_tagging&oldid=1275774341
# Unlintable Unlintable
>            license: CC BY-SA 4.0
# Unlintable Unlintable
>            -->
# Unlintable Unlintable
>            Part   - of - speech tagging
# Unlintable N!PrSg . P  . N!PrSg NSg/V
>
#
> In corpus linguistics , part   - of - speech tagging ( POS tagging or PoS tagging or
# P  N!PrSg N!PrSg      . N!PrSg . P  . N!PrSg NSg/V   . NSg NSg/V   C  NSg NSg/V   C
> POST   ) , also called grammatical tagging is  the process of marking up a word   in a
# N!PrSg . . R    V!X    J           NSg/V   VLX D   N!PrSg  P  V!X     P  D N!PrSg P  D
> text   ( corpus ) as corresponding to a particular part   of speech , based on both its
# N!PrSg . N!PrSg . C  J             P  D J          N!PrSg P  N!PrSg . V!X   P  C    ISg
> definition and its context . A simplified form   of this is  commonly taught to
# N!PrSg     C   ISg N!PrSg  . D V!X        N!PrSg P  D    VLX R        V!X    P
> school - age    children , in the identification of words  as nouns , verbs  , adjectives ,
# N!PrSg . N!PrSg N!PrPl   . P  D   N!PrSg         P  N!PrPl P  NPl/V . N!PrPl . NPl/V      .
> adverbs , etc.
# NPl/V   . N!Pr
>
#
> Once performed by hand   , POS tagging is  now done in the context of computational
# R    V!X       P  N!PrSg . NSg NSg/V   VLX R   V!X  P  D   N!PrSg  P  J
> linguistics , using algorithms which associate discrete terms  , as well as hidden
# N!PrSg      . V!X   N!PrPl     I     V!X       J        N!PrPl . R  R    P  V!X
> parts  of speech , by a set    of descriptive tags  . POS - tagging algorithms fall   into
# N!PrPl P  N!PrSg . P  D N!PrSg P  J           NPl/V . NSg . NSg/V   N!PrPl     N!PrSg P
> two distinctive groups : rule  - based and stochastic . E. Brill's tagger , one       of the
# NSg NSg/J       N!PrPl . NPrSg . V!X   C   J          . ?  ?       NSg    . NSg/I/V/J P  D
> first and most widely used English POS - taggers , employs rule  - based algorithms .
# J     C   R    R      V!X  NPrSg   NSg . NPl     . NPl/V   NPrSg . V!X   N!PrPl     .
>
#
> Principle
# N!PrSg
>
#
> Part   - of - speech tagging is  harder than just having a list   of words  and their
# N!PrSg . P  . N!PrSg NSg/V   VLX J      P    R    V!X    D N!PrSg P  N!PrPl C   I
> parts  of speech , because some words  can represent more than one       part   of speech
# N!PrPl P  N!PrSg . C       D    N!PrPl VX  V!X       R    P    NSg/I/V/J N!PrSg P  N!PrSg
> at different times  , and because some parts  of speech are complex . This is  not
# P  J         N!PrPl . C   C       D    N!PrPl P  N!PrSg VX  J       . I    VLX NSg/C
> rare — in natural languages ( as opposed to many artificial languages ) , a large
# J    . P  J       N!PrPl    . C  V!X     P  J    J          N!PrPl    . . D J
> percentage of word   - forms  are ambiguous . For example , even " dogs   " , which is
# N!PrSg     P  N!PrSg . N!PrPl VX  J         . P   N!PrSg  . R    . N!PrPl . . I     VLX
> usually thought of as just a plural noun   , can also be a verb   :
# R       V!X     P  P  R    D J      N!PrSg . VX  R    VX D N!PrSg .
>
#
> The sailor dogs   the hatch .
# D   NSg    N!PrPl D   NSg/V .
>
#
> Correct grammatical tagging will reflect that " dogs   " is  here used as a verb   , not
# J       J           NSg/V   VX   V!X     I    . N!PrPl . VLX R    V!X  P  D N!PrSg . NSg/C
> as the more common plural noun   . Grammatical context is  one       way    to determine
# P  D   R    J      J      N!PrSg . J           N!PrSg  VLX NSg/I/V/J N!PrSg P  V!X
> this ; semantic analysis can also be used to infer that " sailor " and " hatch "
# I    . J        N!PrSg   VX  R    VX V!X  P  J     I    . NSg    . C   . NSg/V .
> implicate " dogs   " as 1 ) in the nautical context and 2 ) an action applied to the
# NSg/V     . N!PrPl . C  # . P  D   J        N!PrSg  C   # . D  N!PrSg V!X     P  D
> object " hatch " ( in this context , " dogs   " is  a nautical term   meaning " fastens ( a
# N!PrSg . NSg/V . . P  D    N!PrSg  . . N!PrPl . VLX D J        N!PrSg N!PrSg  . V       . D
> watertight door   ) securely " ) .
# J          N!PrSg . R        . . .
>
#
> Tag    sets
# N!PrSg V!X
>
#
> Schools commonly teach that there are 9 parts  of speech in English : noun   , verb   ,
# N!PrPl  R        V!X   C    I     V!X # N!PrPl P  N!PrSg P  NPrSg   . N!PrSg . N!PrSg .
> article , adjective , preposition , pronoun , adverb , conjunction , and interjection .
# N!PrSg  . N!PrSg    . NSg/V       . N!PrSg  . N!PrSg . NSg/V       . C   N!PrSg       .
> However , there are clearly many more categories and sub     - categories . For nouns ,
# R       . I     V!X R       J    R    N!PrPl     C   NSg/V/P . N!PrPl     . P   NPl/V .
> the plural , possessive , and singular forms  can be distinguished . In many
# D   J      . NSg/J      . C   N!PrSg   N!PrPl VX  VX J             . P  J
> languages words  are also marked for their " case   " ( role   as subject , object ,
# N!PrPl    N!PrPl VX  R    V!X    P   I     . N!PrSg . . N!PrSg P  N!PrSg  . N!PrSg .
> etc. ) , grammatical gender , and so on ; while verbs  are marked for tense , aspect ,
# N!Pr . . J           N!PrSg . C   R  P  . C     N!PrPl VX  V!X    P   J     . N!PrSg .
> and other things . In some tagging systems , different inflections of the same
# C   J     N!PrPl . P  D    NSg/V   N!PrPl  . J         NPl         P  D   J
> root   word   will get different parts  of speech , resulting in a large number of
# N!PrSg N!PrSg VX   V!X J         N!PrPl P  N!PrSg . V!X       P  D J     N!PrSg P
> tags  . For example , NN for singular common nouns , NNS for plural common nouns , NP
# NPl/V . P   N!PrSg  . ?  P   N!PrSg   J      NPl/V . ?   P   J      J      NPl/V . NPrSg
> for singular proper nouns ( see the POS tags  used in the Brown Corpus ) . Other
# P   N!PrSg   J      NPl/V . V!X D   NSg NPl/V V!X  P  D   J     N!PrSg . . J
> tagging systems use a smaller number of tags  and ignore fine differences or
# NSg/V   N!PrPl  V!X D J       N!PrSg P  NPl/V C   V!X    J    N!PrSg      C
> model  them as features somewhat independent from part   - of - speech .
# N!PrSg I    P  N!PrPl   R        J           P    N!PrSg . P  . N!PrSg .
>
#
> In part   - of - speech tagging by computer , it  is  typical to distinguish from 50 to
# P  N!PrSg . P  . N!PrSg NSg/V   P  N!PrSg   . ISg VLX J       P  V!X         P    #  P
> 150 separate parts  of speech for English . Work   on stochastic methods for tagging
# #   J        N!PrPl P  N!PrSg P   NPrSg   . N!PrSg P  J          N!PrPl  P   NSg/V
> Koine Greek ( DeRose 1990 ) has used over 1 , 000 parts  of speech and found that
# ?     J     . ?      #    . VX  V!X  P    # . #   N!PrPl P  N!PrSg C   V!X   C
> about as many words  were ambiguous in that language as in English . A
# P     P  J    N!PrPl VX   J         P  D    N!PrSg   P  P  NPrSg   . D
> morphosyntactic descriptor in the case   of morphologically rich languages is
# ?               NSg        P  D   N!PrSg P  ?               J    N!PrPl    VLX
> commonly expressed using very short mnemonics , such as Ncmsan for Category = Noun   ,
# R        V!X       V!X   R    J     NPl       . J    P  ?      P   N!PrSg   . N!PrSg .
> Type   = common , Gender = masculine , Number = singular , Case   = accusative , Animate
# N!PrSg . J      . N!PrSg . NSg/J     . N!PrSg . N!PrSg   . N!PrSg . NSg/J      . V/J
> = no .
# . D  .
>
#
> The most popular " tag    set " for POS tagging for American English is  probably the
# D   R    J       . N!PrSg V!X . P   NSg NSg/V   P   J        NPrSg   VLX R        D
> Penn tag    set , developed in the Penn Treebank project . It  is  largely similar to
# NPr  N!PrSg V!X . V!X       P  D   NPr  ?        N!PrSg  . ISg VLX R       J       P
> the earlier Brown Corpus and LOB   Corpus tag    sets , though much smaller . In
# D   R       J     N!PrSg C   NSg/V N!PrSg N!PrSg V!X  . C      J    J       . P
> Europe , tag    sets from the Eagles Guidelines see wide use and include versions
# NPr    . N!PrSg V!X  P    D   NPl/V  N!PrPl     V!X J    V!X C   V!X     N!PrPl
> for multiple languages .
# P   J        N!PrPl    .
>
#
> POS tagging work   has been done in a variety of languages , and the set    of POS
# NSg NSg/V   N!PrSg VX  VX   V!X  P  D N!PrSg  P  N!PrPl    . C   D   N!PrSg P  NSg
> tags  used varies greatly with language . Tags  usually are designed to include
# NPl/V V!X  V!X    R       P    N!PrSg   . NPl/V R       VX  V!X      P  V!X
> overt morphological distinctions , although this leads to inconsistencies such as
# J     J             N!PrPl       . C        D    V!X   P  NPl             J    P
> case   - marking for pronouns but not   nouns in English , and much larger
# N!PrSg . V!X     P   NPl/V    C   NSg/C NPl/V P  NPrSg   . C   J    J
> cross - language differences . The tag    sets for heavily inflected languages such as
# NPrSg . N!PrSg   N!PrSg      . D   N!PrSg V!X  P   R       V/J       N!PrPl    J    P
> Greek and Latin can be very large ; tagging words  in agglutinative languages such
# J     C   J     VX  VX R    J     . NSg/V   N!PrPl P  ?             N!PrPl    J
> as Inuit languages may be virtually impossible . At the other extreme , Petrov et
# P  J     N!PrPl    VX  VX R         J          . P  D   J     J       . ?      ?
> al. have proposed a " universal " tag    set , with 12 categories ( for example , no
# ?   V!X  V!X      D . J         . N!PrSg V!X . P    #  N!PrPl     . P   N!PrSg  . D
> subtypes of nouns , verbs  , punctuation , and so on ) . Whether a very small set    of
# NPl      P  NPl/V . N!PrPl . N!PrSg      . C   R  P  . . C       D R    J     N!PrSg P
> very broad tags  or a much larger set    of more precise ones   is  preferable , depends
# R    J     NPl/V C  D J    J      N!PrSg P  R    J       N!PrPl VLX J          . V!X
> on the purpose at hand   . Automatic tagging is  easier on smaller tag    - sets .
# P  D   N!PrSg  P  N!PrSg . J         NSg/V   VLX J      P  J       N!PrSg . V!X  .
>
#
> History
# N!PrSg
>
#
> The Brown Corpus
# D   J     N!PrSg
>
#
> Research on part   - of - speech tagging has been closely tied to corpus linguistics .
# N!PrSg   P  N!PrSg . P  . N!PrSg NSg/V   VX  VX   R       V!X  P  N!PrSg N!PrSg      .
> The first major corpus of English for computer analysis was the Brown Corpus
# D   J     J     N!PrSg P  NPrSg   P   N!PrSg   N!PrSg   VX  D   J     N!PrSg
> developed at Brown University by Henry Kučera and W. Nelson Francis , in the
# V!X       P  J     NPrSg      P  NPrSg ?      C   ?  NPrSg  NPr     . P  D
> mid     - 1960s . It  consists of about 1 , 000 , 000 words  of running English prose text   ,
# NSg/J/P . #d    . ISg V!X      P  R     # . #   . #   N!PrPl P  V!X     NPrSg   NSg/V N!PrSg .
> made up of 500 samples from randomly chosen publications . Each sample is  2 , 000
# V!X  P  P  #   N!PrPl  P    R        V!X    N!PrPl       . D    N!PrSg VLX # . #
> or more words  ( ending at the first sentence - end    after 2 , 000 words  , so that the
# C  R    N!PrPl . N!PrSg P  D   J     N!PrSg   . N!PrSg P     # . #   N!PrPl . R  I    D
> corpus contains only complete sentences ) .
# N!PrSg V!X      R    J        N!PrPl    . .
>
#
> The Brown Corpus was painstakingly " tagged " with part   - of - speech markers over
# D   J     N!PrSg VX  R             . V/J    . P    N!PrSg . P  . N!PrSg N!PrPl  P
> many years  . A first approximation was done with a program by Greene and Rubin ,
# J    N!PrPl . D J     N!PrSg        VX  V!X  P    D N!PrSg  P  NPr    C   NPr   .
> which consisted of a huge handmade list   of what categories could co    - occur at
# I     V!X       P  D J    NSg/J    N!PrSg P  I    N!PrPl     VX    NPrSg . V!X   P
> all . For example , article then noun   can occur , but article then verb   ( arguably )
# D   . P   N!PrSg  . N!PrSg  R    N!PrSg VX  V!X   . C   N!PrSg  R    N!PrSg . R        .
> cannot . The program got about 70 % correct . Its results were repeatedly reviewed
# NSg/V  . D   N!PrSg  V!X P     #  . J       . ISg N!PrPl  VX   R          V!X
> and corrected by hand   , and later users  sent in errata so that by the late 70 s
# C   V!X       P  N!PrSg . C   R     N!PrPl V!X  P  NSg    R  I    P  D   J    #  ?
> the tagging was nearly perfect ( allowing for some cases  on which even human
# D   NSg/V   VX  R      J       . V!X      P   D    N!PrPl P  I     R    J
> speakers might not   agree ) .
# W?       VX    NSg/C V!X   . .
>
#
> This corpus has been used for innumerable studies of word   - frequency and of
# D    N!PrSg VX  VX   V!X  P   J           N!PrPl  P  N!PrSg . N!PrSg    C   P
> part   - of - speech and inspired the development of similar " tagged " corpora in many
# N!PrSg . P  . N!PrSg C   V!X      D   N!PrSg      P  J       . V/J    . N!PrPl  P  J
> other languages . Statistics derived by analyzing it  formed the basis  for most
# J     N!PrPl    . N!PrPl     V!X     C  V!X       ISg V!X    D   N!PrSg P   R
> later part   - of - speech tagging systems , such as CLAWS  and VOLSUNGA . However , by
# R     N!PrSg . P  . N!PrSg NSg/V   N!PrPl  . J    P  N!PrPl C   ?        . R       . P
> this time   ( 2005 ) it  has been superseded by larger corpora such as the 100
# D    N!PrSg . #    . ISg VX  VX   V/J        P  J      N!PrPl  J    P  D   #
> million word   British National Corpus , even though larger corpora are rarely so
# N       N!PrSg J       J        N!PrSg . R    C      J      N!PrPl  VX  R      R
> thoroughly curated .
# R          V!X     .
>
#
> For some time   , part   - of - speech tagging was considered an inseparable part   of
# P   D    N!PrSg . N!PrSg . P  . N!PrSg NSg/V   VX  V!X        D  NSg/J       N!PrSg P
> natural language processing , because there are certain cases  where the correct
# J       N!PrSg   N!Pr       . C       I     V!X J       N!PrPl R     D   J
> part   of speech cannot be decided without understanding the semantics or even the
# N!PrSg P  N!PrSg NSg/V  VX V!X     P       N!PrSg        D   N!PrSg    C  R    D
> pragmatics of the context . This is  extremely expensive , especially because
# N!PrPl     P  D   N!PrSg  . I    VLX R         J         . R          C
> analyzing the higher levels is  much harder when multiple part   - of - speech
# V!X       D   J      N!PrPl VLX J    J      R    J        N!PrSg . P  . N!PrSg
> possibilities must be considered for each word   .
# N!PrPl        VX   VX V!X        P   D    N!PrSg .
>
#
> Use of hidden Markov models
# V!X C  V!X    NPr    N!PrPl
>
#
> In the mid     - 1980s , researchers in Europe began to use hidden Markov models ( HMMs )
# P  D   NSg/J/P . #d    . N!Pr        P  NPr    V!X   P  V!X V!X    NPr    N!PrPl . ?    .
> to disambiguate parts  of speech , when working to tag the Lancaster - Oslo - Bergen
# P  V            N!PrPl P  N!PrSg . R    V!X     P  V!X D   NPr       . NPr  . NPr
> Corpus of British English . HMMs involve counting cases  ( such as from the Brown
# N!PrSg P  J       NPrSg   . ?    V!X     N!Pr     N!PrPl . J    P  P    D   J
> Corpus ) and making a table  of the probabilities of certain sequences . For
# N!PrSg . C   V!X    D N!PrSg P  D   NPl           P  J       N!PrPl    . P
> example , once you've seen an article such as ' the ' , perhaps the next word   is  a
# N!PrSg  . R    W?     V!X  D  N!PrSg  J    P  . D   . . R       D   J    N!PrSg VLX D
> noun   40 % of the time   , an adjective 40 % , and a number 20 % . Knowing this , a
# N!PrSg #  . P  D   N!PrSg . D  N!PrSg    #  . . C   D N!PrSg #  . . V!X     I    . D
> program can decide that " can " in " the can " is  far more likely to be a noun   than
# N!PrSg  VX  V!X    I    . VX  . P  . I   VX  . VLX R   R    J      P  VX D N!PrSg P
> a verb   or a modal . The same method can , of course , be used to benefit from
# D N!PrSg C  D NSg/J . D   J    N!PrSg VX  . P  N!PrSg . VX V!X  P  N!PrSg  P
> knowledge about the following words  .
# N!PrSg    P     D   V!X       N!PrPl .
>
#
> More advanced ( " higher - order  " ) HMMs learn the probabilities not   only of pairs
# R    J        . . J      . N!PrSg . . ?    V!X   D   NPl           NSg/C R    P  N!PrPl
> but triples or even larger sequences . So , for example , if you've just seen a
# C   NPl/V   C  R    J      N!PrPl    . R  . P   N!PrSg  . C  W?     R    V!X  D
> noun   followed by a verb   , the next item   may be very likely a preposition ,
# N!PrSg V!X      P  D N!PrSg . D   J    N!PrSg VX  VX R    J      D NSg/V       .
> article , or noun   , but much less likely another verb   .
# N!PrSg  . C  N!PrSg . C   J    R    J      D       N!PrSg .
>
#
> When several ambiguous words  occur together , the possibilities multiply .
# R    J       J         N!PrPl V!X   R        . D   N!PrPl        V!X      .
> However , it  is  easy to enumerate every combination and to assign a relative
# R       . ISg VLX J    P  V         D     N!PrSg      C   P  V!X    D J
> probability to each one    , by multiplying together the probabilities of each
# N!PrSg      P  D    N!PrSg . C  V!X         R        D   NPl           P  D
> choice in turn . The combination with the highest probability is  then chosen . The
# N!PrSg C  V!X  . D   N!PrSg      P    D   J       N!PrSg      VLX R    V!X    . D
> European group  developed CLAWS  , a tagging program that did exactly this and
# J        N!PrSg V!X       N!PrPl . D NSg/V   N!PrSg  I    VX  R       D    C
> achieved accuracy in the 93 – 95 % range  .
# V!X      N!PrSg   P  D   #  . #  . N!PrSg .
>
#
> Eugene Charniak points out in Statistical techniques for natural language
# NPr    ?        N!PrPl P   P  J           N!PrPl     P   J       N!PrSg
> parsing ( 1997 ) that merely assigning the most common tag    to each known word   and
# V       . #    . I    R      V!X       D   R    J      N!PrSg P  D    V!X   N!PrSg C
> the tag    " proper noun   " to all unknowns will approach 90 % accuracy because many
# D   N!PrSg . J      N!PrSg . P  D   N!PrPl   VX   N!PrSg   #  . N!PrSg   C       J
> words  are unambiguous , and many others only rarely represent their less - common
# N!PrPl VX  J           . C   J    N!PrPl R    R      V!X       I     R    . J
> parts  of speech .
# N!PrPl P  N!PrSg .
>
#
> CLAWS  pioneered the field  of HMM - based part   of speech tagging but was quite
# N!PrPl V/J       D   N!PrSg P  V   . V!X   N!PrSg P  N!PrSg NSg/V   C   VX  R
> expensive since it  enumerated all possibilities . It  sometimes had to resort to
# J         C     ISg V/J        D   N!PrPl        . ISg R         V!X P  N!PrSg P
> backup methods when there were simply too many options ( the Brown Corpus
# N!PrSg N!PrPl  R    I     V!X  R      R   J    N!PrPl  . D   J     N!PrSg
> contains a case   with 17 ambiguous words  in a row    , and there are words  such as
# V!X      D N!PrSg P    #  J         N!PrPl P  D N!PrSg . C   I     V!X N!PrPl J    P
> " still " that can represent as many as 7 distinct parts  of speech .
# . R     . I    VX  V!X       P  J    P  # J        N!PrPl P  N!PrSg .
>
#
> HMMs underlie the functioning of stochastic taggers and are used in various
# ?    V        D   N!Pr        P  J          NPl     C   VX  V!X  P  J
> algorithms one       of the most widely used being the bi    - directional inference
# N!PrPl     NSg/I/V/J P  D   R    R      V!X  VX    D   NSg/J . NSg/J       N!PrSg
> algorithm .
# NSg       .
>
#
> Dynamic programming methods
# J       N!PrSg      N!PrPl
>
#
> In 1987 , Steven DeRose and Kenneth W. Church independently developed dynamic
# P  #    . NPr    ?      C   NPr     ?  NPrSg  R             V!X       J
> programming algorithms to solve the same problem in vastly less time   . Their
# N!PrSg      N!PrPl     P  V!X   D   J    N!PrSg  P  R      R    N!PrSg . I
> methods were similar to the Viterbi algorithm known for some time   in other
# N!PrPl  VX   J       P  D   ?       NSg       V!X   P   D    N!PrSg P  J
> fields . DeRose used a table  of pairs  , while Church used a table  of triples and a
# N!PrPl . ?      V!X  D N!PrSg P  N!PrPl . C     NPrSg  V!X  D N!PrSg P  NPl/V   C   D
> method of estimating the values for triples that were rare or nonexistent in the
# N!PrSg P  V          D   N!PrPl P   NPl/V   I    VX   J    C  NSg/J       P  D
> Brown Corpus ( an actual measurement of triple probabilities would require a much
# J     N!PrSg . D  J      NSg         P  J      NPl           VX    V!X     D J
> larger corpus ) . Both methods achieved an accuracy of over 95 % . DeRose's 1990
# J      N!PrSg . . C    N!PrPl  V!X      D  N!PrSg   P  P    #  . . ?        #
> dissertation at Brown University included analyses of the specific error  types  ,
# N!PrSg       P  J     NPrSg      V!X      N!PrSg   P  D   J        N!PrSg N!PrPl .
> probabilities , and other related data   , and replicated his work   for Greek , where
# NPl           . C   J     J       N!PrSg . C   V/J        ISg N!PrSg P   J     . R
> it  proved similarly effective .
# ISg V/J    R         J         .
>
#
> These findings were surprisingly disruptive to the field  of natural language
# D     N!PrSg   VX   R            J          P  D   N!PrSg P  J       N!PrSg
> processing . The accuracy reported was higher than the typical accuracy of very
# N!Pr       . D   N!PrSg   V!X      VX  J      P    D   J       N!PrSg   P  R
> sophisticated algorithms that integrated part   of speech choice with many higher
# J             N!PrPl     I    V!X        N!PrSg P  N!PrSg N!PrSg P    J    J
> levels of linguistic analysis : syntax , morphology , semantics , and so on . CLAWS  ,
# N!PrPl P  J          N!PrSg   . N!PrSg . N!PrSg     . N!PrSg    . C   R  P  . N!PrPl .
> DeRose's and Church's methods did fail for some of the known cases  where
# ?        C   N$       N!PrPl  VX  V!X  P   D    P  D   V!X   N!PrPl R
> semantics is  required , but those proved negligibly rare . This convinced many in
# N!PrSg    VLX V!X      . C   D     V/J    R          J    . D    V!X       J    P
> the field  that part   - of - speech tagging could usefully be separated from the other
# D   N!PrSg I    N!PrSg . P  . N!PrSg NSg/V   VX    R        VX J         P    D   J
> levels of processing ; this , in turn , simplified the theory and practice of
# N!PrPl P  N!Pr       . D    . C  V!X  . V!X        D   N!PrSg C   N!PrSg   P
> computerized language analysis and encouraged researchers to find ways   to
# V/J          N!PrSg   N!PrSg   C   V!X        N!Pr        P  V!X  N!PrPl P
> separate other pieces as well . Markov Models became the standard method for the
# J        J     N!PrPl R  R    . NPr    N!PrPl V!X    D   J        N!PrSg P   D
> part   - of - speech assignment .
# N!PrSg . P  . N!PrSg NSg        .
>
#
> Unsupervised taggers
# V/J          NPl
>
#
> The methods already discussed involve working from a pre    - existing corpus to
# D   N!PrPl  R       V!X       V!X     V!X     P    D N!PrSg . V!X      N!PrSg P
> learn tag    probabilities . It  is  , however , also possible to bootstrap using
# V!X   N!PrSg NPl           . ISg VLX . R       . R    J        P  NSg/V     V!X
> " unsupervised " tagging . Unsupervised tagging techniques use an untagged corpus
# . V/J          . NSg/V   . V/J          NSg/V   N!PrPl     V!X D  ?        N!PrSg
> for their training data   and produce the tagset by induction . That is  , they
# P   I     N!PrSg   N!PrSg C   V!X     D   NSg    P  NSg       . I    VLX . IPl
> observe patterns in word   use , and derive part   - of - speech categories themselves .
# V!X     N!PrPl   P  N!PrSg V!X . C   NSg/V  N!PrSg . P  . N!PrSg N!PrPl     I          .
> For example , statistics readily reveal that " the " , " a " , and " an " occur in
# P   N!PrSg  . N!PrPl     R       V!X    I    . D   . . . D . . C   . D  . V!X   P
> similar contexts , while " eat " occurs in very different ones   . With sufficient
# J       N!PrPl   . C     . V!X . V!X    P  R    J         N!PrPl . P    J
> iteration , similarity classes of words  emerge that are remarkably similar to
# NSg       . NSg        N!PrPl  P  N!PrPl V!X    C    VX  R          J       P
> those human linguists would expect ; and the differences themselves sometimes
# D     J     N!PrPl    VX    V!X    . C   D   N!PrSg      I          R
> suggest valuable new insights .
# V!X     J        J   N!PrPl   .
>
#
> These two categories can be further subdivided into rule  - based , stochastic , and
# D     NSg N!PrPl     VX  VX J       V/J        P    NPrSg . V!X   . J          . C
> neural approaches .
# J      N!PrPl     .
>
#
> Other taggers and methods
# J     NPl     C   N!PrPl
>
#
> Some current major algorithms for part   - of - speech tagging include the Viterbi
# D    J       J     N!PrPl     P   N!PrSg . P  . N!PrSg NSg/V   V!X     D   ?
> algorithm , Brill tagger , Constraint Grammar , and the Baum - Welch algorithm ( also
# NSg       . NSg/J NSg    . N!PrSg     N!PrSg  . C   D   NPr  . ?     NSg       . R
> known as the forward - backward algorithm ) . Hidden Markov model  and visible Markov
# V!X   P  D   R       . J        NSg       . . V!X    NPr    N!PrSg C   J       NPr
> model  taggers can both be implemented using the Viterbi algorithm . The
# N!PrSg NPl     VX  C    VX V!X         V!X   D   ?       NSg       . D
> rule  - based Brill tagger is  unusual in that it  learns a set    of rule  patterns , and
# NPrSg . V!X   NSg/J NSg    VLX J       P  I    ISg NPl/V  D N!PrSg P  NPrSg N!PrPl   . C
> then applies those patterns rather than optimizing a statistical quantity .
# R    V       D     N!PrPl   R      P    V          D J           N!PrSg   .
>
#
> Many machine learning methods have also been applied to the problem of POS
# J    N!PrSg  N!Pr     N!PrPl  V!X  R    VX   V!X     P  D   N!PrSg  P  NSg
> tagging . Methods such as SVM , maximum entropy classifier , perceptron , and
# NSg/V   . N!PrPl  J    P  ?   . J       NSg     NSg        . N          . C
> nearest - neighbor have all been tried , and most can achieve accuracy above
# J       . N!PrSg   V!X  D   VX   V!X   . C   R    VX  V!X     N!PrSg   P
> 95 % . [ citation needed ]
# #  . . . N!PrSg   V!X    .
>
#
> A direct comparison of several methods is  reported ( with references ) at the ACL
# D J      N!PrSg     P  J       N!PrPl  VLX V!X      . P    N!PrPl     . P  D   NSg
> Wiki   . This comparison uses the Penn tag    set on some of the Penn Treebank data   ,
# N!PrSg . D    N!PrSg     V!X  D   NPr  N!PrSg V!X P  D    P  D   NPr  ?        N!PrSg .
> so the results are directly comparable . However , many significant taggers are
# R  D   N!PrPl  VX  R        J          . R       . J    J           NPl     VX
> not   included ( perhaps because of the labor     involved in reconfiguring them for
# NSg/C V!X      . R       C       P  D   N!PrSg/Am V!X      P  V             I    P
> this particular dataset ) . Thus , it  should not   be assumed that the results
# D    J          NSg     . . R    . ISg VX     NSg/C VX V!X     C    D   N!PrPl
> reported here are the best that can be achieved with a given approach ; nor even
# V!X      R    VX  D   J    C    VX  VX V!X      P    D V!X   N!PrSg   . C   R
> the best that have been achieved with a given approach .
# D   J    C    V!X  VX   V!X      P    D V!X   N!PrSg   .
>
#
> In 2014 , a paper  reporting using the structure regularization method for
# P  #    . D N!PrSg V!X       V!X   D   N!PrSg    NSg            N!PrSg P
> part   - of - speech tagging , achieving 97.36 % on a standard benchmark dataset .
# N!PrSg . P  . N!PrSg NSg/V   . V!X       #     . P  D J        NSg/V     NSg     .
