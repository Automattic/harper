> <!--
# Unlintable
>            source: https://en.wikipedia.org/w/index.php?title=Part-of-speech_tagging&oldid=1275774341
# Unlintable Unlintable
>            license: CC BY-SA 4.0
# Unlintable Unlintable
>            -->
# Unlintable Unlintable
>            Part     - of - speech  tagging
# Unlintable NSg/V/J+ . P  . N🅪Sg/V+ NSg/V
>
#
> In      corpus linguistics , part     - of - speech  tagging ( POS  tagging or    PoS  tagging or
# NPr/J/P NSg+   NᴹSg+       . NSg/V/J+ . P  . N🅪Sg/V+ NSg/V   . NSg+ NSg/V   NPr/C NSg+ NSg/V   NPr/C
> POST      ) , also called grammatical tagging is the process of marking up        a   word   in      a
# NPr🅪/V/P+ . . W?   V/J    J           NSg/V   VL D   NSg/V   P  NSg/V   NSg/V/J/P D/P NSg/V+ NPr/J/P D/P
> text    ( corpus ) as    corresponding to a   particular part    of speech  , based on  both   its
# N🅪Sg/V+ . NSg+   . NSg/R NSg/V/J       P  D/P NSg/J      NSg/V/J P  N🅪Sg/V+ . V/J   J/P I/C/Dq ISg/D$+
> definition and its     context . A   simplified form  of this    is commonly taught to
# NSg        V/C ISg/D$+ N🅪Sg/V+ . D/P V/J        NSg/V P  I/Ddem+ VL R        V      P
> school - age     children , in      the identification of words  as    nouns , verbs  , adjectives ,
# NSg/V  . N🅪Sg/V+ NPl+     . NPr/J/P D   NSg            P  NPl/V+ NSg/R NPl/V . NPl/V+ . NPl/V      .
> adverbs , etc.
# NPl/V   . +
>
#
> Once  performed by      hand   , POS  tagging is now       done    in      the context of computational
# NSg/C V/J       NSg/J/P NSg/V+ . NSg+ NSg/V   VL NPr/V/J/C NSg/V/J NPr/J/P D   N🅪Sg/V  P  J
> linguistics , using algorithms which associate discrete terms  , as    well    as    hidden
# NᴹSg+       . V     NPl+       I/C+  NSg/V/J+  J        NPl/V+ . NSg/R NSg/V/J NSg/R V/J
> parts of speech  , by      a   set     of descriptive tags   . POS  - tagging algorithms fall   into
# NPl/V P  N🅪Sg/V+ . NSg/J/P D/P NPr/V/J P  NSg/J       NPl/V+ . NSg+ . NSg/V   NPl+       NSg/V+ P
> two distinctive groups : rule   - based and stochastic . E. Brill's tagger , one       of the
# NSg NSg/J       NPl/V+ . NSg/V+ . V/J   V/C J          . ?  ?       NSg    . NSg/I/V/J P  D
> first   and most       widely used English   POS  - taggers , employs rule   - based algorithms .
# NSg/V/J V/C NSg/I/J/Dq R      V/J  NPr🅪/V/J+ NSg+ . NPl     . NPl/V   NSg/V+ . V/J   NPl+       .
>
#
> Principle
# N🅪Sg/V+
>
#
> Part     - of - speech  tagging is harder than just having a   list  of words  and their
# NSg/V/J+ . P  . N🅪Sg/V+ NSg/V   VL JC     C/P  V/J  V      D/P NSg/V P  NPl/V+ V/C D$+
> parts of speech  , because some     words  can    represent more         than one       part    of speech
# NPl/V P  N🅪Sg/V+ . C/P     I/J/R/Dq NPl/V+ NPr/VX V         NPr/I/V/J/Dq C/P  NSg/I/V/J NSg/V/J P  N🅪Sg/V+
> at    different times  , and because some     parts of speech  are complex . This    is not
# NSg/P NSg/J     NPl/V+ . V/C C/P     I/J/R/Dq NPl/V P  N🅪Sg/V+ V   NSg/V/J . I/Ddem+ VL NSg/C
> rare    — in      natural languages ( as    opposed to many        artificial languages ) , a   large
# NSg/V/J . NPr/J/P NSg/J+  NPl/V+    . NSg/R V/J     P  NSg/I/J/Dq+ J+         NPl/V+    . . D/P NSg/J
> percentage of word   - forms  are ambiguous . For example , even    " dogs   " , which is
# N🅪Sg       P  NSg/V+ . NPl/V+ V   J         . C/P NSg/V+  . NSg/V/J . NPl/V+ . . I/C+  VL
> usually thought of as    just a    plural noun   , can    also be     a    verb   :
# R       NSg/V   P  NSg/R V/J  D/P+ NSg/J+ NSg/V+ . NPr/VX W?   NSg/VX D/P+ NSg/V+ .
>
#
> The sailor dogs   the hatch  .
# D+  NSg+   NPl/V+ D+  NSg/V+ .
>
#
> Correct grammatical tagging will   reflect that          " dogs   " is here    used as    a   verb   , not
# NSg/V/J J           NSg/V   NPr/VX V       NSg/I/C/Ddem+ . NPl/V+ . VL NSg/J/R V/J  NSg/R D/P NSg/V+ . NSg/C
> as    the more         common  plural noun   . Grammatical context is one       way   to determine
# NSg/R D   NPr/I/V/J/Dq NSg/V/J NSg/J  NSg/V+ . J+          N🅪Sg/V+ VL NSg/I/V/J NSg/J P  V
> this    ; semantic analysis can    also be     used to infer that          " sailor " and " hatch "
# I/Ddem+ . NSg/J+   N🅪Sg+    NPr/VX W?   NSg/VX V/J  P  V     NSg/I/C/Ddem+ . NSg+   . V/C . NSg/V .
> implicate " dogs   " as    1 ) in      the nautical context and 2 ) an  action   applied to the
# NSg/V     . NPl/V+ . NSg/R # . NPr/J/P D   J        N🅪Sg/V+ V/C # . D/P NSg/V/J+ V/J     P  D
> object " hatch " ( in      this   context , " dogs   " is a   nautical term     meaning   " fastens ( a
# NSg/V+ . NSg/V . . NPr/J/P I/Ddem N🅪Sg/V+ . . NPl/V+ . VL D/P J        NSg/V/J+ N🅪Sg/V/J+ . V       . D/P
> watertight door   ) securely " ) .
# J          NSg/V+ . R        . . .
>
#
> Tag    sets
# NSg/V+ NPl/V
>
#
> Schools commonly teach that         there are 9 parts of speech in      English  : noun   , verb   ,
# NPl/V+  R        NSg/V NSg/I/C/Ddem +     V   # NPl/V P  N🅪Sg/V NPr/J/P NPr🅪/V/J . NSg/V+ . NSg/V+ .
> article , adjective , preposition , pronoun , adverb , conjunction , and interjection .
# NSg/V+  . NSg/V/J+  . NSg/V       . NSg/V+  . NSg/V+ . NSg/V+      . V/C NSg+         .
> However , there are clearly many        more          categories and sub     - categories . For nouns ,
# C       . +     V   R       NSg/I/J/Dq+ NPr/I/V/J/Dq+ NPl+       V/C NSg/V/P . NPl+       . C/P NPl/V .
> the plural , possessive , and singular forms  can    be     distinguished . In      many
# D   NSg/J  . NSg/J      . V/C NSg/J    NPl/V+ NPr/VX NSg/VX V/J           . NPr/J/P NSg/I/J/Dq+
> languages words  are also marked for their " case   " ( role as    subject  , object ,
# NPl/V+    NPl/V+ V   W?   V/J    C/P D$+   . NPr/V+ . . NSg  NSg/R NSg/V/J+ . NSg/V+ .
> etc. ) , grammatical gender   , and so        on  ; while     verbs  are marked for tense   , aspect ,
# +    . . J+          NSg/V/J+ . V/C NSg/I/J/C J/P . NSg/V/C/P NPl/V+ V   V/J    C/P NSg/V/J . NSg/V+ .
> and other    things . In      some     tagging systems , different inflections of the same
# V/C NSg/V/J+ NPl/V+ . NPr/J/P I/J/R/Dq NSg/V   NPl+    . NSg/J     NPl         P  D   I/J
> root   word   will   get   different parts of speech  , resulting in      a   large number   of
# NPr/V+ NSg/V+ NPr/VX NSg/V NSg/J     NPl/V P  N🅪Sg/V+ . V         NPr/J/P D/P NSg/J NSg/V/JC P
> tags   . For example , NN for singular common  nouns , NNS for plural common  nouns , NP
# NPl/V+ . C/P NSg/V+  . ?  C/P NSg/J    NSg/V/J NPl/V . ?   C/P NSg/J  NSg/V/J NPl/V . NPr
> for singular proper nouns ( see   the POS  tags   used in      the Brown    Corpus ) . Other
# C/P NSg/J    NSg/J  NPl/V . NSg/V D   NSg+ NPl/V+ V/J  NPr/J/P D   NPr🅪/V/J NSg+   . . NSg/V/J
> tagging systems use   a   smaller number   of tags   and ignore fine    differences or
# NSg/V   NPl+    NSg/V D/P NSg/JC  NSg/V/JC P  NPl/V+ V/C V      NSg/V/J NSg/V       NPr/C
> model    them     as    features somewhat independent from part     - of - speech  .
# NSg/V/J+ NSg/IPl+ NSg/R NPl/V+   NSg/I    NSg/J       P    NSg/V/J+ . P  . N🅪Sg/V+ .
>
#
> In      part     - of - speech  tagging by      computer , it       is typical to distinguish from 50 to
# NPr/J/P NSg/V/J+ . P  . N🅪Sg/V+ NSg/V   NSg/J/P NSg/V+   . NPr/ISg+ VL NSg/J   P  V           P    #  P
> 150 separate parts of speech for English   . Work   on  stochastic methods for tagging
# #   NSg/V/J  NPl/V P  N🅪Sg/V C/P NPr🅪/V/J+ . N🅪Sg/V J/P J          NPl/V+  C/P NSg/V
> Koine Greek   ( DeRose 1990 ) has used over    1 , 000 parts of speech  and found that
# ?     NPr/V/J . ?      #    . V   V/J  NSg/J/P # . #   NPl/V P  N🅪Sg/V+ V/C NSg/V NSg/I/C/Ddem
> about as    many       words  were  ambiguous in      that         language as    in      English   . A
# J/P   NSg/R NSg/I/J/Dq NPl/V+ NSg/V J         NPr/J/P NSg/I/C/Ddem N🅪Sg/V+  NSg/R NPr/J/P NPr🅪/V/J+ . D/P
> morphosyntactic descriptor in      the case  of morphologically rich    languages is
# ?               NSg        NPr/J/P D   NPr/V P  ?               NPr/V/J NPl/V+    VL
> commonly expressed using very short     mnemonics , such  as    Ncmsan for Category = Noun   ,
# R        V/J       V     J/R  NPr/V/J/P NPl       . NSg/I NSg/R ?      C/P NSg+     . NSg/V+ .
> Type   = common  , Gender   = masculine , Number    = singular , Case   = accusative , Animate
# NSg/V+ . NSg/V/J . NSg/V/J+ . NSg/J     . NSg/V/JC+ . NSg/J    . NPr/V+ . NSg/J      . V/J
> = no    .
# . NPr/P .
>
#
> The most       popular " tag   set     " for POS  tagging for American English   is probably the
# D   NSg/I/J/Dq NSg/J   . NSg/V NPr/V/J . C/P NSg+ NSg/V   C/P NPr/J    NPr🅪/V/J+ VL R        D
> Penn tag    set     , developed in      the Penn Treebank project . It       is largely similar to
# NPr+ NSg/V+ NPr/V/J . V/J       NPr/J/P D   NPr+ ?        NSg/V+  . NPr/ISg+ VL R       NSg/J   P
> the earlier Brown    Corpus and LOB   Corpus tag    sets  , though much       smaller . In
# D   JC      NPr🅪/V/J NSg    V/C NSg/V NSg+   NSg/V+ NPl/V . V/C    NSg/I/J/Dq NSg/JC  . NPr/J/P
> Europe , tag    sets  from the Eagles Guidelines see   wide  use    and include versions
# NPr+   . NSg/V+ NPl/V P    D   NPl/V  NPl+       NSg/V NSg/J NSg/V+ V/C NSg/V   NPl/V+
> for multiple languages .
# C/P NSg/J/Dq NPl/V+    .
>
#
> POS  tagging work    has been  done    in      a   variety of languages , and the set     of POS
# NSg+ NSg/V   N🅪Sg/V+ V   NSg/V NSg/V/J NPr/J/P D/P NSg     P  NPl/V+    . V/C D   NPr/V/J P  NSg+
> tags   used varies greatly with language . Tags   usually are designed to include
# NPl/V+ V/J  NPl/V  R       P    N🅪Sg/V+  . NPl/V+ R       V   V/J      P  NSg/V
> overt  morphological distinctions , although this   leads to inconsistencies such  as
# NSg/J+ J+            NPl+         . C        I/Ddem NPl/V P  NPl             NSg/I NSg/R
> case   - marking for pronouns but     not   nouns in      English   , and much       larger
# NPr/V+ . NSg/V   C/P NPl/V    NSg/C/P NSg/C NPl/V NPr/J/P NPr🅪/V/J+ . V/C NSg/I/J/Dq JC
> cross      - language differences . The tag    sets  for heavily inflected languages such  as
# NPr/V/J/P+ . N🅪Sg/V+  NSg/V+      . D+  NSg/V+ NPl/V C/P R       V/J       NPl/V+    NSg/I NSg/R
> Greek   and Latin can    be     very large ; tagging words  in      agglutinative languages such
# NPr/V/J V/C NPr/J NPr/VX NSg/VX J/R  NSg/J . NSg/V   NPl/V+ NPr/J/P ?             NPl/V+    NSg/I
> as    Inuit languages may    be     virtually impossible . At    the other   extreme , Petrov et
# NSg/R NPr/J NPl/V+    NPr/VX NSg/VX R         NSg/J      . NSg/P D   NSg/V/J NSg/J   . ?      ?
> al. have   proposed a   " universal " tag    set     , with 12 categories ( for example , no
# ?   NSg/VX V/J      D/P . NSg/J     . NSg/V+ NPr/V/J . P    #  NPl+       . C/P NSg/V+  . NPr/P
> subtypes of nouns , verbs  , punctuation , and so        on  ) . Whether a   very small   set     of
# NPl      P  NPl/V . NPl/V+ . NᴹSg+       . V/C NSg/I/J/C J/P . . I/C     D/P J/R  NPr/V/J NPr/V/J P
> very broad tags   or    a   much       larger set     of more         precise ones   is preferable , depends
# J/R  NSg/J NPl/V+ NPr/C D/P NSg/I/J/Dq JC     NPr/V/J P  NPr/I/V/J/Dq V/J+    NPl/V+ VL W?         . NPl/V
> on  the purpose at    hand   . Automatic tagging is easier on  smaller tag    - sets  .
# J/P D   NSg/V+  NSg/P NSg/V+ . NSg/J     NSg/V   VL NSg/JC J/P NSg/JC  NSg/V+ . NPl/V .
>
#
> History
# N🅪Sg+
>
#
> The Brown     Corpus
# D+  NPr🅪/V/J+ NSg+
>
#
> Research on  part     - of - speech  tagging has been  closely tied to corpus linguistics .
# NᴹSg/V   J/P NSg/V/J+ . P  . N🅪Sg/V+ NSg/V   V   NSg/V R       V/J  P  NSg    NᴹSg+       .
> The first   major   corpus of English  for computer analysis was the Brown    Corpus
# D   NSg/V/J NPr/V/J NSg    P  NPr🅪/V/J C/P NSg/V+   N🅪Sg+    V   D   NPr🅪/V/J NSg
> developed at    Brown    University by      Henry Kučera and W. Nelson Francis , in      the
# V/J       NSg/P NPr🅪/V/J NSg+       NSg/J/P NPr+  ?      V/C ?  NPr+   NPr+    . NPr/J/P D
> mid      - 1960s . It       consists of about 1 , 000 , 000 words of running   English   prose text    ,
# NSg/J/P+ . #d    . NPr/ISg+ NPl/V    P  J/P   # . #   . #   NPl/V P  NSg/V/J/P NPr🅪/V/J+ NSg/V N🅪Sg/V+ .
> made up        of 500 samples from randomly chosen   publications . Each sample is 2 , 000
# V    NSg/V/J/P P  #   NPl/V+  P    R        NᴹSg/V/J NPl+         . Dq+  NSg/V+ VL # . #
> or    more         words  ( ending at    the first   sentence - end    after 2 , 000 words  , so        that         the
# NPr/C NPr/I/V/J/Dq NPl/V+ . NSg/V  NSg/P D   NSg/V/J NSg/V+   . NSg/V+ P     # . #   NPl/V+ . NSg/I/J/C NSg/I/C/Ddem D
> corpus contains only  complete sentences ) .
# NSg+   V        J/R/C NSg/V/J  NPl/V+    . .
>
#
> The Brown     Corpus was painstakingly " tagged " with part     - of - speech  markers over
# D+  NPr🅪/V/J+ NSg+   V   R             . V/J    . P    NSg/V/J+ . P  . N🅪Sg/V+ NPl/V   NSg/J/P
> many        years . A    first    approximation was done    with a    program by      Greene and Rubin ,
# NSg/I/J/Dq+ NPl+  . D/P+ NSg/V/J+ N🅪Sg+         V   NSg/V/J P    D/P+ NPr/V+  NSg/J/P NPr    V/C NPr   .
> which consisted of a   huge handmade list  of what   categories could  co       - occur at
# I/C+  V/J       P  D/P J    NSg/J    NSg/V P  NSg/I+ NPl+       NSg/VX NPr/I/V+ . V     NSg/P
> all          . For example , article then    noun   can    occur , but     article then    verb   ( arguably )
# NSg/I/J/C/Dq . C/P NSg/V+  . NSg/V+  NSg/J/C NSg/V+ NPr/VX V     . NSg/C/P NSg/V+  NSg/J/C NSg/V+ . R        .
> cannot . The program got about 70 % correct . Its     results were  repeatedly reviewed
# NSg/V  . D+  NPr/V+  V   J/P   #  . NSg/V/J . ISg/D$+ NPl/V+  NSg/V R          V/J
> and corrected by      hand   , and later users sent  in      errata so        that          by      the late  70 s
# V/C V/J       NSg/J/P NSg/V+ . V/C JC    NPl+  NSg/V NPr/J/P NSg    NSg/I/J/C NSg/I/C/Ddem+ NSg/J/P D   NSg/J #  ?
> the tagging was nearly perfect ( allowing for some     cases  on  which even    human
# D   NSg/V   V   R      NSg/V/J . V        C/P I/J/R/Dq NPl/V+ J/P I/C+  NSg/V/J NSg/V/J+
> speakers might     not   agree ) .
# +        NᴹSg/VX/J NSg/C V     . .
>
#
> This    corpus has been  used for innumerable studies of word   - frequency and of
# I/Ddem+ NSg+   V   NSg/V V/J  C/P J           NPl/V   P  NSg/V+ . NSg       V/C P
> part     - of - speech  and inspired the development of similar " tagged " corpora in      many
# NSg/V/J+ . P  . N🅪Sg/V+ V/C V/J      D   N🅪Sg        P  NSg/J   . V/J    . NPl+    NPr/J/P NSg/I/J/Dq
> other   languages . Statistics derived by      analyzing it       formed the basis for most
# NSg/V/J NPl/V+    . NPl/V+     V/J     NSg/J/P V         NPr/ISg+ V/J    D+  NSg+  C/P NSg/I/J/Dq
> later part     - of - speech  tagging systems , such  as    CLAWS  and VOLSUNGA . However , by
# JC    NSg/V/J+ . P  . N🅪Sg/V+ NSg/V   NPl+    . NSg/I NSg/R NPl/V+ V/C ?        . C       . NSg/J/P
> this    time      ( 2005 ) it       has been  superseded by      larger corpora such  as    the 100
# I/Ddem+ N🅪Sg/V/J+ . #    . NPr/ISg+ V   NSg/V V/J        NSg/J/P JC     NPl+    NSg/I NSg/R D   #
> million word   British National Corpus , even    though larger corpora are rarely so
# NSg     NSg/V+ NPr/J   NSg/J    NSg+   . NSg/V/J V/C    JC     NPl+    V   R      NSg/I/J/C
> thoroughly curated .
# R          V/J     .
>
#
> For some     time      , part     - of - speech  tagging was considered an  inseparable part    of
# C/P I/J/R/Dq N🅪Sg/V/J+ . NSg/V/J+ . P  . N🅪Sg/V+ NSg/V   V   V/J        D/P NSg/J       NSg/V/J P
> natural language processing , because there are certain cases  where the correct
# NSg/J   N🅪Sg/V+  V+         . C/P     +     V   I/J     NPl/V+ NSg/C D   NSg/V/J
> part    of speech  cannot be     decided without understanding the semantics or    even    the
# NSg/V/J P  N🅪Sg/V+ NSg/V  NSg/VX NSg/V/J C/P     NᴹSg/V/J+     D   NPl+      NPr/C NSg/V/J D
> pragmatics of the context . This    is extremely expensive , especially because
# NPl        P  D   N🅪Sg/V+ . I/Ddem+ VL R         J         . R          C/P
> analyzing the higher  levels is much       harder when    multiple part    - of - speech
# V         D+  NSg/JC+ NPl/V+ VL NSg/I/J/Dq JC     NSg/I/C NSg/J/Dq NSg/V/J . P  . N🅪Sg/V+
> possibilities must  be     considered for each word   .
# NPl+          NSg/V NSg/VX V/J        C/P Dq+  NSg/V+ .
>
#
> Use   of hidden Markov models
# NSg/V P  V/J    NPr    NPl/V+
>
#
> In      the mid      - 1980s , researchers in      Europe began to use   hidden Markov models ( HMMs )
# NPr/J/P D   NSg/J/P+ . #d    . NPl         NPr/J/P NPr+   V     P  NSg/V V/J    NPr    NPl/V+ . ?    .
> to disambiguate parts of speech  , when    working to tag   the Lancaster - Oslo - Bergen
# P  V            NPl/V P  N🅪Sg/V+ . NSg/I/C V       P  NSg/V D   NPr       . NPr+ . NPr+
> Corpus of British English   . HMMs involve counting cases  ( such  as    from the Brown
# NSg    P  NPr/J   NPr🅪/V/J+ . ?    V       V        NPl/V+ . NSg/I NSg/R P    D   NPr🅪/V/J
> Corpus ) and making a   table of the probabilities of certain sequences . For
# NSg+   . V/C NSg/V  D/P NSg/V P  D   NPl           P  I/J     NPl/V+    . C/P
> example , once  you've seen  an  article such  as    ' the ' , perhaps the next    word   is a
# NSg/V+  . NSg/C W?     NSg/V D/P NSg/V+  NSg/I NSg/R . D   . . NSg     D   NSg/J/P NSg/V+ VL D/P
> noun   40 % of the time      , an  adjective 40 % , and a   number    20 % . Knowing   this    , a
# NSg/V+ #  . P  D   N🅪Sg/V/J+ . D/P NSg/V/J+  #  . . V/C D/P NSg/V/JC+ #  . . NSg/V/J/P I/Ddem+ . D/P+
> program can    decide that          " can    " in      " the can    " is far     more         likely to be     a   noun  than
# NPr/V+  NPr/VX V      NSg/I/C/Ddem+ . NPr/VX . NPr/J/P . D+  NPr/VX . VL NSg/V/J NPr/I/V/J/Dq NSg/J  P  NSg/VX D/P NSg/V C/P
> a    verb   or    a   modal . The same method can    , of course , be     used to benefit from
# D/P+ NSg/V+ NPr/C D/P NSg/J . D+  I/J+ NSg/V+ NPr/VX . P  NSg/V+ . NSg/VX V/J  P  NSg/V   P
> knowledge about the following words  .
# NᴹSg+     J/P   D+  NSg/V/J/P NPl/V+ .
>
#
> More         advanced ( " higher - order " ) HMMs learn the probabilities not   only  of pairs
# NPr/I/V/J/Dq V/J      . . NSg/JC . NSg/V . . ?    NSg/V D   NPl+          NSg/C J/R/C P  NPl/V+
> but     triples or    even    larger sequences . So        , for example , if    you've just seen  a
# NSg/C/P NPl/V   NPr/C NSg/V/J JC     NPl/V+    . NSg/I/J/C . C/P NSg/V+  . NSg/C W?     V/J  NSg/V D/P
> noun   followed by      a   verb   , the next    item   may    be     very likely a   preposition ,
# NSg/V+ V/J      NSg/J/P D/P NSg/V+ . D   NSg/J/P NSg/V+ NPr/VX NSg/VX J/R  NSg/J  D/P NSg/V       .
> article , or    noun   , but     much       less    likely another verb   .
# NSg/V+  . NPr/C NSg/V+ . NSg/C/P NSg/I/J/Dq V/J/C/P NSg/J  I/D     NSg/V+ .
>
#
> When    several ambiguous words  occur together , the possibilities multiply .
# NSg/I/C J/Dq+   J+        NPl/V+ V     J        . D+  NPl+          NSg/V    .
> However , it       is easy    to enumerate every combination and to assign a   relative
# C       . NPr/ISg+ VL NSg/V/J P  V         Dq+   N🅪Sg+       V/C P  NSg/V  D/P NSg/J
> probability to each one        , by      multiplying together the probabilities of each
# NSg+        P  Dq   NSg/I/V/J+ . NSg/J/P V           J        D   NPl           P  Dq
> choice in      turn  . The combination with the highest probability is then    chosen   . The
# NSg/J+ NPr/J/P NSg/V . D   N🅪Sg        P    D+  JS+     NSg+        VL NSg/J/C NᴹSg/V/J . D+
> European group  developed CLAWS  , a   tagging program that          did exactly this   and
# NSg/J+   NSg/V+ V/J       NPl/V+ . D/P NSg/V   NPr/V+  NSg/I/C/Ddem+ V   R       I/Ddem V/C
> achieved accuracy in      the 93 – 95 % range  .
# V/J      N🅪Sg+    NPr/J/P D   #  . #  . NSg/V+ .
>
#
> Eugene Charniak points out         in      Statistical techniques for natural language
# NPr+   ?        NPl/V+ NSg/V/J/R/P NPr/J/P J           NPl        C/P NSg/J+  N🅪Sg/V+
> parsing ( 1997 ) that          merely assigning the most       common  tag    to each known word   and
# V       . #    . NSg/I/C/Ddem+ R      V         D   NSg/I/J/Dq NSg/V/J NSg/V+ P  Dq   V/J   NSg/V+ V/C
> the tag    " proper noun   " to all          unknowns will   approach 90 % accuracy because many
# D   NSg/V+ . NSg/J  NSg/V+ . P  NSg/I/J/C/Dq NPl/V+   NPr/VX NSg/V+   #  . N🅪Sg+    C/P     NSg/I/J/Dq
> words  are unambiguous , and many       others only  rarely represent their less    - common
# NPl/V+ V   J           . V/C NSg/I/J/Dq NPl/V+ J/R/C R      V         D$+   V/J/C/P . NSg/V/J
> parts of speech  .
# NPl/V P  N🅪Sg/V+ .
>
#
> CLAWS  pioneered the field of HMM - based part    of speech  tagging but     was quite
# NPl/V+ V/J       D   NSg/V P  V   . V/J   NSg/V/J P  N🅪Sg/V+ NSg/V   NSg/C/P V   NSg
> expensive since it       enumerated all          possibilities . It       sometimes had to resort to
# J         C/P   NPr/ISg+ V/J        NSg/I/J/C/Dq NPl+          . NPr/ISg+ R         V   P  NSg/V  P
> backup methods when    there were  simply too many       options ( the Brown     Corpus
# NSg/J  NPl/V+  NSg/I/C +     NSg/V R      W?  NSg/I/J/Dq NPl/V   . D+  NPr🅪/V/J+ NSg+
> contains a   case   with 17 ambiguous words in      a    row    , and there are words  such  as
# V        D/P NPr/V+ P    #  J         NPl/V NPr/J/P D/P+ NSg/V+ . V/C +     V   NPl/V+ NSg/I NSg/R
> " still   " that          can    represent as    many       as    7 distinct parts of speech  .
# . NSg/V/J . NSg/I/C/Ddem+ NPr/VX V         NSg/R NSg/I/J/Dq NSg/R # V/J      NPl/V P  N🅪Sg/V+ .
>
#
> HMMs underlie the functioning of stochastic taggers and are used in      various
# ?    V        D   V+          P  J          NPl     V/C V   V/J  NPr/J/P J
> algorithms one       of the most       widely used being    the bi    - directional inference
# NPl+       NSg/I/V/J P  D   NSg/I/J/Dq R      V/J  N🅪Sg/V/C D   NSg/J . NSg/J       NSg+
> algorithm .
# NSg       .
>
#
> Dynamic programming methods
# NSg/J+  NᴹSg/V+     NPl/V+
>
#
> In      1987 , Steven DeRose and Kenneth W. Church independently developed dynamic
# NPr/J/P #    . NPr+   ?      V/C NPr+    ?  NPr/V+ R             V/J       NSg/J
> programming algorithms to solve the same problem in      vastly less    time      . Their
# NᴹSg/V+     NPl+       P  NSg/V D   I/J  NSg/J+  NPr/J/P R      V/J/C/P N🅪Sg/V/J+ . D$+
> methods were  similar to the Viterbi algorithm known for some     time      in      other
# NPl/V+  NSg/V NSg/J   P  D   ?       NSg       V/J   C/P I/J/R/Dq N🅪Sg/V/J+ NPr/J/P NSg/V/J
> fields   . DeRose used a   table of pairs  , while     Church used a   table of triples and a
# NPrPl/V+ . ?      V/J  D/P NSg/V P  NPl/V+ . NSg/V/C/P NPr/V+ V/J  D/P NSg/V P  NPl/V   V/C D/P
> method of estimating the values for triples that          were  rare    or    nonexistent in      the
# NSg/V  P  V          D   NPl/V+ C/P NPl/V   NSg/I/C/Ddem+ NSg/V NSg/V/J NPr/C NSg/J       NPr/J/P D
> Brown    Corpus ( an  actual measurement of triple  probabilities would require a   much
# NPr🅪/V/J NSg+   . D/P NSg/J  NSg         P  NSg/V/J NPl+          VX    NSg/V   D/P NSg/I/J/Dq
> larger corpus ) . Both   methods achieved an  accuracy of over    95 % . DeRose's 1990
# JC     NSg+   . . I/C/Dq NPl/V+  V/J      D/P N🅪Sg+    P  NSg/J/P #  . . ?        #
> dissertation at    Brown    University included analyses    of the specific error  types  ,
# NSg+         NSg/P NPr🅪/V/J NSg+       V/J      NPl/V/Au/Br P  D   NSg/J    NSg/V+ NPl/V+ .
> probabilities , and other   related data  , and replicated his     work    for Greek   , where
# NPl+          . V/C NSg/V/J J       N🅪Pl+ . V/C V/J        ISg/D$+ N🅪Sg/V+ C/P NPr/V/J . NSg/C
> it       proved similarly effective .
# NPr/ISg+ V/J    R         NSg/J     .
>
#
> These   findings were  surprisingly disruptive to the field of natural language
# I/Ddem+ NSg+     NSg/V R            J          P  D   NSg/V P  NSg/J+  N🅪Sg/V+
> processing . The accuracy reported was higher than the typical accuracy of very
# V+         . D+  N🅪Sg+    V/J      V   NSg/JC C/P  D   NSg/J   N🅪Sg     P  J/R
> sophisticated algorithms that          integrated part    of speech  choice with many       higher
# V/J+          NPl+       NSg/I/C/Ddem+ V/J        NSg/V/J P  N🅪Sg/V+ NSg/J+ P    NSg/I/J/Dq NSg/JC
> levels of linguistic analysis : syntax , morphology , semantics , and so        on  . CLAWS  ,
# NPl/V  P  J          N🅪Sg     . NᴹSg+  . NᴹSg+      . NPl+      . V/C NSg/I/J/C J/P . NPl/V+ .
> DeRose's and Church's methods did fail    for some     of the known cases  where
# ?        V/C NSg$     NPl/V+  V   NSg/V/J C/P I/J/R/Dq P  D   V/J   NPl/V+ NSg/C
> semantics is required , but     those  proved negligibly rare    . This   convinced many       in
# NPl+      VL V/J      . NSg/C/P I/Ddem V/J    R          NSg/V/J . I/Ddem V/J       NSg/I/J/Dq NPr/J/P
> the field  that          part     - of - speech  tagging could  usefully be     separated from the other
# D+  NSg/V+ NSg/I/C/Ddem+ NSg/V/J+ . P  . N🅪Sg/V+ NSg/V   NSg/VX R        NSg/VX V/J       P    D   NSg/V/J
> levels of processing ; this    , in      turn  , simplified the theory and practice of
# NPl/V  P  V+         . I/Ddem+ . NPr/J/P NSg/V . V/J        D   NSg    V/C NSg/V    P
> computerized language analysis and encouraged researchers to find  ways to
# V/J          N🅪Sg/V+  N🅪Sg+    V/C V/J        NPl+        P  NSg/V NPl+ P
> separate other   pieces as    well    . Markov Models became the standard method for the
# NSg/V/J  NSg/V/J NPl/V+ NSg/R NSg/V/J . NPr    NPl/V+ V      D   NSg/J    NSg/V+ C/P D
> part     - of - speech  assignment .
# NSg/V/J+ . P  . N🅪Sg/V+ NSg+       .
>
#
> Unsupervised taggers
# V/J          NPl
>
#
> The methods already discussed involve working from a    pre      - existing corpus to
# D+  NPl/V+  W?      V/J       V       V       P    D/P+ NSg/V/P+ . V        NSg+   P
> learn tag    probabilities . It       is , however , also possible to bootstrap using
# NSg/V NSg/V+ NPl+          . NPr/ISg+ VL . C       . W?   NSg/J    P  NSg/V     V
> " unsupervised " tagging . Unsupervised tagging techniques use   an  untagged corpus
# . V/J          . NSg/V   . V/J          NSg/V   NPl+       NSg/V D/P J        NSg+
> for their training data  and produce the tagset by      induction . That          is , they
# C/P D$+   NᴹSg/V+  N🅪Pl+ V/C NSg/V   D   NSg    NSg/J/P NSg       . NSg/I/C/Ddem+ VL . IPl+
> observe patterns in      word   use   , and derive part     - of - speech  categories themselves .
# NSg/V   NPl/V+   NPr/J/P NSg/V+ NSg/V . V/C NSg/V  NSg/V/J+ . P  . N🅪Sg/V+ NPl+       IPl+       .
> For example , statistics readily reveal that          " the " , " a   " , and " an  " occur in
# C/P NSg/V+  . NPl/V+     R       NSg/V  NSg/I/C/Ddem+ . D   . . . D/P . . V/C . D/P . V     NPr/J/P
> similar contexts , while     " eat " occurs in      very different ones   . With sufficient
# NSg/J+  NPl/V+   . NSg/V/C/P . V   . V      NPr/J/P J/R  NSg/J+    NPl/V+ . P    J
> iteration , similarity classes of words  emerge that          are remarkably similar to
# NSg       . NSg        NPl/V   P  NPl/V+ NSg/V  NSg/I/C/Ddem+ V   R          NSg/J   P
> those  human   linguists would expect ; and the differences themselves sometimes
# I/Ddem NSg/V/J NPl+      VX    V      . V/C D   NSg/V+      IPl+       R
> suggest valuable new     insights .
# V       NSg/J    NSg/V/J NPl+     .
>
#
> These   two  categories can    be     further subdivided into rule   - based , stochastic , and
# I/Ddem+ NSg+ NPl+       NPr/VX NSg/VX V/J     V/J        P    NSg/V+ . V/J   . J          . V/C
> neural approaches .
# J      NPl/V+     .
>
#
> Other   taggers and methods
# NSg/V/J NPl     V/C NPl/V+
>
#
> Some     current major   algorithms for part     - of - speech  tagging include the Viterbi
# I/J/R/Dq NSg/J   NPr/V/J NPl        C/P NSg/V/J+ . P  . N🅪Sg/V+ NSg/V   NSg/V   D   ?
> algorithm , Brill tagger , Constraint Grammar , and the Baum - Welch algorithm ( also
# NSg       . NSg/J NSg    . NSg+       N🅪Sg/V+ . V/C D   NPr  . ?     NSg       . W?
> known as    the forward - backward algorithm ) . Hidden Markov model    and visible Markov
# V/J   NSg/R D   NSg/V/J . NSg/J    NSg       . . V/J    NPr    NSg/V/J+ V/C J       NPr
> model    taggers can    both   be     implemented using the Viterbi algorithm . The
# NSg/V/J+ NPl     NPr/VX I/C/Dq NSg/VX V/J         V     D   ?       NSg       . D+
> rule   - based Brill tagger is unusual in      that         it       learns a   set     of rule   patterns , and
# NSg/V+ . V/J   NSg/J NSg    VL NSg/J   NPr/J/P NSg/I/C/Ddem NPr/ISg+ NPl/V  D/P NPr/V/J P  NSg/V+ NPl/V+   . V/C
> then    applies those  patterns rather  than optimizing a   statistical quantity .
# NSg/J/C V       I/Ddem NPl/V+   NPr/V/J C/P  V          D/P J           NSg+     .
>
#
> Many        machine learning methods have   also been  applied to the problem of POS
# NSg/I/J/Dq+ NSg/V+  V+       NPl/V+  NSg/VX W?   NSg/V V/J     P  D   NSg/J   P  NSg+
> tagging . Methods such  as    SVM , maximum entropy classifier , perceptron , and
# NSg/V   . NPl/V+  NSg/I NSg/R ?   . NSg/J   NSg     NSg        . NSg        . V/C
> nearest - neighbor    have   all          been  tried , and most       can    achieve accuracy above
# JS      . NSg/V/J/Am+ NSg/VX NSg/I/J/C/Dq NSg/V V/J   . V/C NSg/I/J/Dq NPr/VX V       N🅪Sg+    NSg/J/P
> 95 % . [ citation needed ]
# #  . . . NSg+     V/J    .
>
#
> A   direct comparison of several methods is reported ( with references ) at    the ACL
# D/P V/J    NSg        P  J/Dq+   NPl/V+  VL V/J      . P    NPl/V+     . NSg/P D   NSg
> Wiki   . This    comparison uses  the Penn tag    set     on  some     of the Penn Treebank data  ,
# NSg/V+ . I/Ddem+ NSg+       NPl/V D+  NPr+ NSg/V+ NPr/V/J J/P I/J/R/Dq P  D   NPr+ ?        N🅪Pl+ .
> so        the results are directly comparable . However , many       significant taggers are
# NSg/I/J/C D   NPl/V+  V   R/C      NSg/J      . C       . NSg/I/J/Dq NSg/J       NPl     V
> not   included ( perhaps because of the labor        involved in      reconfiguring them     for
# NSg/C V/J      . NSg     C/P     P  D   NPr/V/Am/Au+ V/J      NPr/J/P V             NSg/IPl+ C/P
> this   particular dataset ) . Thus , it       should not   be     assumed that         the results
# I/Ddem NSg/J      NSg     . . NSg  . NPr/ISg+ VX     NSg/C NSg/VX V/J     NSg/I/C/Ddem D+  NPl/V+
> reported here    are the best      that          can    be     achieved with a    given      approach ; nor   even
# V/J      NSg/J/R V   D   NPr/VX/JS NSg/I/C/Ddem+ NPr/VX NSg/VX V/J      P    D/P+ NSg/V/J/P+ NSg/V+   . NSg/C NSg/V/J
> the best       that          have   been  achieved with a    given      approach .
# D+  NPr/VX/JS+ NSg/I/C/Ddem+ NSg/VX NSg/V V/J      P    D/P+ NSg/V/J/P+ NSg/V+   .
>
#
> In      2014 , a    paper     reporting using the structure regularization method for
# NPr/J/P #    . D/P+ N🅪Sg/V/J+ V         V     D   N🅪Sg/V+   N🅪Sg           NSg/V  C/P
> part     - of - speech  tagging , achieving 97.36 % on  a   standard benchmark dataset .
# NSg/V/J+ . P  . N🅪Sg/V+ NSg/V   . V         #     . J/P D/P NSg/J    NSg/V     NSg     .
