> <!--
# Unlintable
>            source: https://en.wikipedia.org/w/index.php?title=Part-of-speech_tagging&oldid=1275774341
# Unlintable Unlintable
>            license: CC BY-SA 4.0
# Unlintable Unlintable
>            -->
# Unlintable Unlintable
>            Part    - of - speech tagging
# Unlintable NSg/V/J . P  . NSg/V+ NSg/V
>
#
> In        corpus linguistics , part    - of - speech tagging ( POS  tagging or      PoS  tagging or
# NPrSg/J/P NSg+   NSg         . NSg/V/J . P  . NSg/V  NSg/V   . NSg+ NSg/V   NPrSg/C NSg+ NSg/V   NPrSg/C
> POST       ) , also called grammatical tagging is the process of marking up        a   word  in        a
# NPrSg/V/P+ . . W?   V/J    J           NSg/V   VL D   NSg/V   P  NSg/V   NSg/V/J/P D/P NSg/V NPrSg/J/P D/P
> text  ( corpus ) as    corresponding to a   particular part    of speech , based on  both its
# NSg/V . NSg+   . NSg/R NSg/V/J       P  D/P NSg/J      NSg/V/J P  NSg/V+ . V/J   J/P I/C  ISg/D$+
> definition and its     context . A   simplified form  of this    is commonly taught to
# NSg        V/C ISg/D$+ NSg/V+  . D/P V/J        NSg/V P  I/Ddem+ VL R        V      P
> school - age   children , in        the identification of words  as    nouns , verbs  , adjectives ,
# NSg/V  . NSg/V NPl      . NPrSg/J/P D   NSg            P  NPl/V+ NSg/R NPl/V . NPl/V+ . NPl/V      .
> adverbs , etc.
# NPl/V   . W?
>
#
> Once  performed by      hand   , POS  tagging is now         done    in        the context of computational
# NSg/C V/J       NSg/J/P NSg/V+ . NSg+ NSg/V   VL NPrSg/V/J/C NSg/V/J NPrSg/J/P D   NSg/V   P  J+
> linguistics , using algorithms which associate discrete terms  , as    well    as    hidden
# NSg+        . V     NPl+       I/C+  NSg/V/J+  J        NPl/V+ . NSg/R NSg/V/J NSg/R V/J
> parts of speech , by      a   set       of descriptive tags   . POS  - tagging algorithms fall  into
# NPl/V P  NSg/V+ . NSg/J/P D/P NPrSg/V/J P  NSg/J+      NPl/V+ . NSg+ . NSg/V   NPl        NSg/V P
> two distinctive groups : rule   - based and  stochastic . E. Brill's tagger , one       of the
# NSg NSg/J       NPl/V+ . NSg/V+ . V/J+  V/C+ J+         . ?  ?       NSg    . NSg/I/V/J P  D
> first   and most    widely used English    POS  - taggers , employs rule   - based algorithms .
# NSg/V/J V/C NSg/I/J R      V/J  NPrSg/V/J+ NSg+ . NPl     . NPl/V   NSg/V+ . V/J   NPl+       .
>
#
> Principle
# NSg/V
>
#
> Part    - of - speech tagging is harder than just having a   list  of words and their
# NSg/V/J . P  . NSg/V  NSg/V   VL J      C/P  V/J  V      D/P NSg/V P  NPl/V V/C D$+
> parts of speech , because some   words  can      represent more        than one       part    of speech
# NPl/V P  NSg/V+ . C/P     I/J/R+ NPl/V+ NPrSg/VX V         NPrSg/I/V/J C/P  NSg/I/V/J NSg/V/J P  NSg/V+
> at    different times  , and because some  parts of speech are complex  . This    is not
# NSg/P NSg/J+    NPl/V+ . V/C C/P     I/J/R NPl/V P  NSg/V+ V+  NSg/V/J+ . I/Ddem+ VL NSg/C
> rare    — in        natural languages ( as    opposed to many    artificial languages ) , a   large
# NSg/V/J . NPrSg/J/P NSg/J   NPl/V+    . NSg/R V/J     P  N/I/J/D J          NPl/V+    . . D/P NSg/J
> percentage of word   - forms  are ambiguous . For example , even    " dogs   " , which is
# NSg        P  NSg/V+ . NPl/V+ V+  J+        . C/P NSg/V+  . NSg/V/J . NPl/V+ . . I/C+  VL
> usually thought of as    just a    plural noun   , can      also be     a   verb   :
# R       NSg/V   P  NSg/R V/J  D/P+ NSg/J+ NSg/V+ . NPrSg/VX W?   NSg/VX D/P NSg/V+ .
>
#
> The sailor dogs  the hatch  .
# D+  NSg    NPl/V D   NSg/V+ .
>
#
> Correct  grammatical tagging will     reflect that        " dogs   " is here    used as    a    verb   , not
# NSg/V/J+ J           NSg/V   NPrSg/VX V       N/I/C/Ddem+ . NPl/V+ . VL NSg/J/R V/J  NSg/R D/P+ NSg/V+ . NSg/C
> as    the more        common  plural noun   . Grammatical context is one       way    to determine
# NSg/R D   NPrSg/I/V/J NSg/V/J NSg/J  NSg/V+ . J           NSg/V+  VL NSg/I/V/J NSg/J+ P  V
> this    ; semantic analysis can      also be     used to infer that        " sailor " and " hatch "
# I/Ddem+ . NSg/J    NSg+     NPrSg/VX W?   NSg/VX V/J  P  J     N/I/C/Ddem+ . NSg+   . V/C . NSg/V .
> implicate " dogs  " as    1 ) in        the nautical context and 2 ) an  action   applied to the
# NSg/V     . NPl/V . NSg/R # . NPrSg/J/P D+  J+       NSg/V+  V/C # . D/P NSg/V/J+ V/J     P  D
> object " hatch " ( in        this    context , " dogs   " is a   nautical term     meaning  " fastens ( a
# NSg/V+ . NSg/V . . NPrSg/J/P I/Ddem+ NSg/V+  . . NPl/V+ . VL D/P J        NSg/V/J+ NSg/V/J+ . V       . D/P
> watertight door   ) securely " ) .
# J          NSg/V+ . R        . . .
>
#
> Tag    sets
# NSg/V+ NPl/V
>
#
> Schools commonly teach that       there are 9 parts of speech in        English   : noun   , verb   ,
# NPl/V+  R        NSg/V N/I/C/Ddem +     V   # NPl/V P  NSg/V+ NPrSg/J/P NPrSg/V/J . NSg/V+ . NSg/V+ .
> article , adjective , preposition , pronoun , adverb , conjunction , and interjection .
# NSg/V+  . NSg/V/J+  . NSg/V       . NSg/V+  . NSg/V+ . NSg/V+      . V/C NSg+         .
> However , there are clearly many    more        categories and sub     - categories . For nouns ,
# C       . +     V   R       N/I/J/D NPrSg/I/V/J NPl+       V/C NSg/V/P . NPl        . C/P NPl/V .
> the plural , possessive , and singular forms  can       be      distinguished . In        many
# D   NSg/J  . NSg/J      . V/C NSg/J    NPl/V+ NPrSg/VX+ NSg/VX+ V/J+          . NPrSg/J/P N/I/J/D+
> languages words  are also marked for their " case     " ( role as    subject , object ,
# NPl/V+    NPl/V+ V   W?   V/J    C/P D$+   . NPrSg/V+ . . NSg  NSg/R NSg/V/J . NSg/V+ .
> etc. ) , grammatical gender   , and so        on  ; while     verbs  are marked for tense   , aspect ,
# +    . . J+          NSg/V/J+ . V/C NSg/I/J/C J/P . NSg/V/C/P NPl/V+ V   V/J    C/P NSg/V/J . NSg/V+ .
> and other    things . In        some   tagging systems , different inflections of the same
# V/C NSg/V/J+ NPl/V+ . NPrSg/J/P I/J/R+ NSg/V   NPl+    . NSg/J     NPl         P  D+  I/J+
> root     word   will     get   different parts of speech , resulting in        a   large number  of
# NPrSg/V+ NSg/V+ NPrSg/VX NSg/V NSg/J     NPl/V P  NSg/V+ . V         NPrSg/J/P D/P NSg/J NSg/V/J P+
> tags   . For example , NN for singular common   nouns , NNS for plural common   nouns , NP
# NPl/V+ . C/P NSg/V+  . ?  C/P NSg/J    NSg/V/J+ NPl/V . ?   C/P NSg/J  NSg/V/J+ NPl/V . NPrSg
> for singular proper nouns ( see   the POS  tags   used in        the Brown      Corpus ) . Other
# C/P NSg/J    NSg/J  NPl/V . NSg/V D+  NSg+ NPl/V+ V/J  NPrSg/J/P D+  NPrSg/V/J+ NSg+   . . NSg/V/J
> tagging systems use   a   smaller number  of tags   and ignore fine    differences or
# NSg/V   NPl+    NSg/V D/P J       NSg/V/J P  NPl/V+ V/C V      NSg/V/J NSg/V       NPrSg/C
> model    them as     features somewhat independent from part    - of - speech .
# NSg/V/J+ N/I+ NSg/R+ NPl/V+   NSg/I    NSg/J       P    NSg/V/J . P  . NSg/V+ .
>
#
> In        part    - of - speech tagging by      computer , it         is typical to distinguish from 50 to
# NPrSg/J/P NSg/V/J . P  . NSg/V  NSg/V   NSg/J/P NSg/V+   . NPrSg/ISg+ VL NSg/J   P  V           P    #  P
> 150 separate parts of speech for English    . Work  on  stochastic methods for tagging
# #   NSg/V/J  NPl/V P  NSg/V  C/P NPrSg/V/J+ . NSg/V J/P J          NPl/V   C/P NSg/V
> Koine Greek     ( DeRose 1990 ) has used over      1 , 000 parts of speech and found that
# ?     NPrSg/V/J . ?      #    . V   V/J  NSg/V/J/P # . #   NPl/V P  NSg/V+ V/C NSg/V N/I/C/Ddem
> about as    many     words  were  ambiguous in        that        language as    in        English    . A
# J/P   NSg/R N/I/J/D+ NPl/V+ NSg/V J         NPrSg/J/P N/I/C/Ddem+ NSg/V+   NSg/R NPrSg/J/P NPrSg/V/J+ . D/P
> morphosyntactic descriptor in        the case    of morphologically rich      languages is
# ?               NSg        NPrSg/J/P D   NPrSg/V P  ?               NPrSg/V/J NPl/V+    VL
> commonly expressed using very short        mnemonics , such  as    Ncmsan for Category = Noun   ,
# R        V/J       V     J    NPrSg/V/J/P+ NPl       . NSg/I NSg/R ?      C/P NSg      . NSg/V+ .
> Type  = common  , Gender  = masculine , Number  = singular , Case    = accusative , Animate
# NSg/V . NSg/V/J . NSg/V/J . NSg/J     . NSg/V/J . NSg/J    . NPrSg/V . NSg/J      . V/J
> = no      .
# . NPrSg/P .
>
#
> The most    popular " tag    set       " for POS  tagging for American English    is probably the
# D   NSg/I/J NSg/J   . NSg/V+ NPrSg/V/J . C/P NSg+ NSg/V   C/P NPrSg/J  NPrSg/V/J+ VL R        D+
> Penn tag    set       , developed in        the Penn Treebank project . It         is largely similar to
# NPr+ NSg/V+ NPrSg/V/J . V/J       NPrSg/J/P D+  NPr+ ?        NSg/V+  . NPrSg/ISg+ VL R       NSg/J   P
> the earlier Brown     Corpus and LOB   Corpus tag    sets  , though much   smaller . In
# D   J       NPrSg/V/J NSg    V/C NSg/V NSg+   NSg/V+ NPl/V . V/C    N/I/J+ J+      . NPrSg/J/P
> Europe , tag    sets  from the Eagles Guidelines see   wide  use    and include versions
# NPr+   . NSg/V+ NPl/V P    D+  NPl/V+ NPl+       NSg/V NSg/J NSg/V+ V/C NSg/V   NPl/V
> for multiple languages .
# C/P NSg/J+   NPl/V+    .
>
#
> POS  tagging work   has been  done    in        a   variety of languages , and the set       of POS
# NSg+ NSg/V   NSg/V+ V   NSg/V NSg/V/J NPrSg/J/P D/P NSg     P  NPl/V+    . V/C D   NPrSg/V/J P  NSg+
> tags   used varies greatly with language . Tags   usually are designed to include
# NPl/V+ V/J  NPl/V  R       P    NSg/V+   . NPl/V+ R       V   V/J      P  NSg/V
> overt morphological distinctions , although this    leads to inconsistencies such  as
# NSg/J J+            NPl+         . C        I/Ddem+ NPl/V P  NPl             NSg/I NSg/R
> case     - marking for pronouns but     not   nouns in        English    , and much  larger
# NPrSg/V+ . NSg/V   C/P NPl/V    NSg/C/P NSg/C NPl/V NPrSg/J/P NPrSg/V/J+ . V/C N/I/J J
> cross        - language differences . The tag    sets  for heavily inflected languages such  as
# NPrSg/V/J/P+ . NSg/V+   NSg/V       . D+  NSg/V+ NPl/V C/P R       V/J       NPl/V+    NSg/I NSg/R
> Greek     and Latin   can      be     very large ; tagging words  in        agglutinative languages such
# NPrSg/V/J V/C NPrSg/J NPrSg/VX NSg/VX J    NSg/J . NSg/V   NPl/V+ NPrSg/J/P ?             NPl/V+    NSg/I
> as    Inuit   languages may      be     virtually impossible . At    the other    extreme , Petrov et
# NSg/R NPrSg/J NPl/V+    NPrSg/VX NSg/VX R+        NSg/J+     . NSg/P D+  NSg/V/J+ NSg/J   . ?      ?
> al. have   proposed a   " universal " tag    set       , with 12 categories ( for example , no
# ?   NSg/VX V/J      D/P . NSg/J     . NSg/V+ NPrSg/V/J . P    #  NPl        . C/P NSg/V+  . NPrSg/P
> subtypes of nouns , verbs  , punctuation , and so        on   ) . Whether a   very small     set       of
# NPl      P  NPl/V . NPl/V+ . NSg+        . V/C NSg/I/J/C J/P+ . . I/C     D/P J    NPrSg/V/J NPrSg/V/J P
> very broad tags  or      a   much  larger set       of more        precise ones   is preferable , depends
# J    NSg/J NPl/V NPrSg/C D/P N/I/J J      NPrSg/V/J P  NPrSg/I/V/J V/J     NPl/V+ VL W?         . NPl/V
> on  the purpose at    hand   . Automatic tagging is easier on  smaller tag    - sets   .
# J/P D+  NSg/V   NSg/P NSg/V+ . NSg/J     NSg/V   VL J      J/P J       NSg/V+ . NPl/V+ .
>
#
> History
# NSg
>
#
> The Brown      Corpus
# D   NPrSg/V/J+ NSg
>
#
> Research on  part    - of - speech tagging has been  closely tied to corpus linguistics .
# NSg/V    J/P NSg/V/J . P  . NSg/V  NSg/V   V   NSg/V R       V/J  P  NSg    NSg+        .
> The first   major     corpus of English    for computer analysis was the Brown     Corpus
# D   NSg/V/J NPrSg/V/J NSg    P  NPrSg/V/J+ C/P NSg/V+   NSg+     V   D   NPrSg/V/J NSg
> developed at    Brown     University by      Henry  Kučera and W. Nelson Francis , in        the
# V/J       NSg/P NPrSg/V/J NSg        NSg/J/P NPrSg+ ?      V/C ?  NPrSg+ NPr+    . NPrSg/J/P D
> mid      - 1960s . It         consists of about 1 , 000 , 000 words of running   English    prose text   ,
# NSg/J/P+ . #d    . NPrSg/ISg+ NPl/V    P  J/P   # . #   . #   NPl/V P  NSg/V/J/P NPrSg/V/J+ NSg/V NSg/V+ .
> made  up        of 500 samples from randomly chosen publications . Each sample is 2 , 000
# NSg/V NSg/V/J/P P  #   NPl/V+  P    R+       V/J    NPl+         . D+   NSg/V+ VL # . #
> or      more        words  ( ending at    the first    sentence - end   after 2 , 000 words  , so        that       the
# NPrSg/C NPrSg/I/V/J NPl/V+ . NSg/V  NSg/P D   NSg/V/J+ NSg/V+   . NSg/V J/P   # . #   NPl/V+ . NSg/I/J/C N/I/C/Ddem D+
> corpus contains only complete sentences ) .
# NSg+   V        W?   NSg/V/J+ NPl/V+    . .
>
#
> The Brown     Corpus was painstakingly " tagged " with part    - of - speech markers over
# D+  NPrSg/V/J NSg    V   R             . V/J    . P    NSg/V/J . P  . NSg/V  NPl/V   NSg/V/J/P
> many     years . A    first    approximation was done    with a   program by      Greene and Rubin ,
# N/I/J/D+ NPl+  . D/P+ NSg/V/J+ NSg+          V   NSg/V/J P    D/P NPrSg/V NSg/J/P NPr    V/C NPr   .
> which consisted of a   huge handmade list  of what   categories could  co         - occur at
# I/C+  V/J       P  D/P J    NSg/J    NSg/V P  NSg/I+ NPl+       NSg/VX NPrSg/I/V+ . V     NSg/P+
> all       . For example , article then    noun   can      occur , but     article then    verb   ( arguably )
# NSg/I/J/C . C/P NSg/V+  . NSg/V+  NSg/J/C NSg/V+ NPrSg/VX V     . NSg/C/P NSg/V+  NSg/J/C NSg/V+ . R        .
> cannot . The program  got about 70 % correct  . Its     results were  repeatedly reviewed
# NSg/V  . D+  NPrSg/V+ V   J/P   #  . NSg/V/J+ . ISg/D$+ NPl/V+  NSg/V R          V/J
> and corrected by      hand   , and later users sent  in        errata so        that        by      the late  70 s
# V/C V/J       NSg/J/P NSg/V+ . V/C J     NPl+  NSg/V NPrSg/J/P NSg    NSg/I/J/C N/I/C/Ddem+ NSg/J/P D   NSg/J #  ?
> the tagging was nearly perfect ( allowing for some  cases  on  which even    human
# D   NSg/V   V   R      NSg/V/J . V        C/P I/J/R NPl/V+ J/P I/C+  NSg/V/J NSg/V/J
> speakers might    not   agree ) .
# +        NSg/VX/J NSg/C V     . .
>
#
> This    corpus has been  used for innumerable studies of word   - frequency and of
# I/Ddem+ NSg    V   NSg/V V/J  C/P J           NPl/V   P  NSg/V+ . NSg       V/C P
> part    - of - speech and inspired the development of similar " tagged " corpora in        many
# NSg/V/J . P  . NSg/V  V/C V/J      D   NSg         P  NSg/J   . V/J    . NPl     NPrSg/J/P N/I/J/D+
> other    languages . Statistics derived by      analyzing it         formed the basis for most
# NSg/V/J+ NPl/V+    . NPl/V+     V/J     NSg/J/P V         NPrSg/ISg+ V/J    D   NSg   C/P NSg/I/J
> later part    - of - speech tagging systems , such  as    CLAWS  and VOLSUNGA . However , by
# J     NSg/V/J . P  . NSg/V  NSg/V   NPl     . NSg/I NSg/R NPl/V+ V/C ?        . C       . NSg/J/P
> this    time     ( 2005 ) it         has been  superseded by      larger corpora such  as    the 100
# I/Ddem+ NSg/V/J+ . #    . NPrSg/ISg+ V   NSg/V V/J        NSg/J/P J      NPl+    NSg/I NSg/R D   #
> million word   British National Corpus , even    though larger corpora are rarely so
# N       NSg/V+ NPrSg/J NSg/J+   NSg+   . NSg/V/J V/C    J+     NPl+    V   R      NSg/I/J/C
> thoroughly curated .
# R+         V/J+    .
>
#
> For some  time    , part    - of - speech tagging was considered an  inseparable part    of
# C/P I/J/R NSg/V/J . NSg/V/J . P  . NSg/V  NSg/V   V   V/J        D/P NSg/J       NSg/V/J P
> natural language processing , because there are certain cases  where the correct
# NSg/J+  NSg/V+   V+         . C/P     +     V   I/J     NPl/V+ NSg/C D   NSg/V/J
> part    of speech cannot be     decided without understanding the semantics or      even    the
# NSg/V/J P  NSg/V+ NSg/V  NSg/VX NSg/V/J C/P     NSg/V/J+      D+  NSg       NPrSg/C NSg/V/J D
> pragmatics of the context . This    is extremely expensive , especially because
# NPl        P  D+  NSg/V+  . I/Ddem+ VL R         J         . R          C/P
> analyzing the higher levels is much  harder when    multiple part    - of - speech
# V         D+  J+     NPl/V+ VL N/I/J J      NSg/I/C NSg/J    NSg/V/J . P  . NSg/V
> possibilities must  be     considered for each word   .
# NPl           NSg/V NSg/VX V/J        C/P D+   NSg/V+ .
>
#
> Use   of hidden Markov models
# NSg/V P  V/J    NPr+   NPl/V
>
#
> In        the mid     - 1980s , researchers in        Europe began to use   hidden Markov models ( HMMs )
# NPrSg/J/P D   NSg/J/P . #d    . W?          NPrSg/J/P NPr+   V     P  NSg/V V/J    NPr    NPl/V+ . ?    .
> to disambiguate parts of speech , when    working to tag   the Lancaster - Oslo - Bergen
# P  V            NPl/V P  NSg/V+ . NSg/I/C V       P  NSg/V D   NPr       . NPr+ . NPr
> Corpus of British  English    . HMMs involve counting cases ( such  as    from the Brown
# NSg    P  NPrSg/J+ NPrSg/V/J+ . ?    V       V        NPl/V . NSg/I NSg/R P    D+  NPrSg/V/J+
> Corpus ) and making a   table of the probabilities of certain sequences . For
# NSg+   . V/C NSg/V  D/P NSg/V P  D   NPl           P  I/J+    NPl/V+    . C/P
> example , once  you've seen  an  article such  as    ' the ' , perhaps the next     word   is a
# NSg/V+  . NSg/C W?     NSg/V D/P NSg/V+  NSg/I NSg/R . D   . . NSg     D+  NSg/J/P+ NSg/V+ VL D/P
> noun  40 % of the time     , an   adjective 40 % , and a    number   20 % . Knowing   this    , a
# NSg/V #  . P  D+  NSg/V/J+ . D/P+ NSg/V/J+  #  . . V/C D/P+ NSg/V/J+ #  . . NSg/V/J/P I/Ddem+ . D/P+
> program  can      decide that        " can      " in        " the can      " is far     more        likely to be     a   noun  than
# NPrSg/V+ NPrSg/VX V      N/I/C/Ddem+ . NPrSg/VX . NPrSg/J/P . D+  NPrSg/VX . VL NSg/V/J NPrSg/I/V/J NSg/J  P  NSg/VX D/P NSg/V C/P
> a   verb  or      a    modal  . The same method can      , of course , be     used to benefit from
# D/P NSg/V NPrSg/C D/P+ NSg/J+ . D+  I/J+ NSg/V+ NPrSg/VX . P  NSg/V+ . NSg/VX V/J  P  NSg/V   P
> knowledge about the following  words .
# NSg/V+    J/P   D+  NSg/V/J/P+ NPl/V .
>
#
> More        advanced ( " higher - order " ) HMMs learn the probabilities not   only of pairs
# NPrSg/I/V/J V/J      . . J      . NSg/V . . ?    NSg/V D+  NPl+          NSg/C W?   P  NPl/V+
> but     triples or      even    larger sequences . So        , for example , if    you've just seen  a
# NSg/C/P NPl/V   NPrSg/C NSg/V/J J      NPl/V+    . NSg/I/J/C . C/P NSg/V+  . NSg/C W?     V/J  NSg/V D/P
> noun  followed by      a    verb   , the next     item   may      be     very likely a   preposition ,
# NSg/V V/J      NSg/J/P D/P+ NSg/V+ . D+  NSg/J/P+ NSg/V+ NPrSg/VX NSg/VX J    NSg/J  D/P NSg/V       .
> article , or      noun   , but     much  less    likely another verb  .
# NSg/V+  . NPrSg/C NSg/V+ . NSg/C/P N/I/J V/J/C/P NSg/J+ I/D     NSg/V .
>
#
> When    several ambiguous words  occur together , the possibilities multiply .
# NSg/I/C J/D     J         NPl/V+ V     J        . D+  NPl           NSg/V+   .
> However , it         is easy    to enumerate every combination and to assign a   relative
# C       . NPrSg/ISg+ VL NSg/V/J P  V         D+    NSg+        V/C P  NSg/V  D/P NSg/J
> probability to each one        , by      multiplying together the probabilities of each
# NSg         P  D+   NSg/I/V/J+ . NSg/J/P V           J        D   NPl           P  D+
> choice in        turn  . The combination with the highest probability is then     chosen . The
# NSg/J+ NPrSg/J/P NSg/V . D   NSg         P    D+  +       NSg+        VL NSg/J/C+ V/J    . D+
> European group  developed CLAWS  , a   tagging program  that        did exactly this    and
# NSg/J+   NSg/V+ V/J       NPl/V+ . D/P NSg/V+  NPrSg/V+ N/I/C/Ddem+ V   R       I/Ddem+ V/C
> achieved accuracy in        the 93 – 95 % range  .
# V/J      NSg+     NPrSg/J/P D   #  . #  . NSg/V+ .
>
#
> Eugene Charniak points out         in        Statistical techniques for natural language
# NPr+   ?        NPl/V+ NSg/V/J/R/P NPrSg/J/P J           NPl        C/P NSg/J   NSg/V+
> parsing ( 1997 ) that        merely assigning the most    common  tag   to each known   word  and
# V       . #    . N/I/C/Ddem+ R      V         D   NSg/I/J NSg/V/J NSg/V P  D+   NSg/V/J NSg/V V/C
> the tag    " proper noun  " to all        unknowns will     approach 90 % accuracy because many
# D   NSg/V+ . NSg/J  NSg/V . P  NSg/I/J/C+ NPl/V+   NPrSg/VX NSg/V    #  . NSg+     C/P     N/I/J/D+
> words  are unambiguous , and many     others only rarely represent their less    - common
# NPl/V+ V   J           . V/C N/I/J/D+ NPl/V+ W?   R      V         D$+   V/J/C/P . NSg/V/J
> parts of speech .
# NPl/V P  NSg/V+ .
>
#
> CLAWS  pioneered the field of HMM - based part    of speech tagging but     was quite
# NPl/V+ V/J       D   NSg/V P  V   . V/J   NSg/V/J P  NSg/V+ NSg/V   NSg/C/P V   NSg
> expensive since it         enumerated all        possibilities . It         sometimes had to resort to
# J         C/P   NPrSg/ISg+ V/J        NSg/I/J/C+ NPl+          . NPrSg/ISg+ R         V   P  NSg/V  P
> backup methods when    there were  simply too many     options ( the Brown      Corpus
# NSg/J  NPl/V+  NSg/I/C +     NSg/V R      W?  N/I/J/D+ NPl/V   . D+  NPrSg/V/J+ NSg+
> contains a   case    with 17 ambiguous words in        a    row    , and there are words  such  as
# V        D/P NPrSg/V P    #  J         NPl/V NPrSg/J/P D/P+ NSg/V+ . V/C +     V   NPl/V+ NSg/I NSg/R
> " still   " that        can      represent as    many    as    7 distinct parts of speech .
# . NSg/V/J . N/I/C/Ddem+ NPrSg/VX V         NSg/R N/I/J/D NSg/R # V/J      NPl/V P  NSg/V+ .
>
#
> HMMs underlie the functioning of stochastic taggers and are used in        various
# ?    V        D   V           P  J          NPl     V/C V   V/J  NPrSg/J/P J
> algorithms one       of the most    widely used being   the bi    - directional inference
# NPl+       NSg/I/V/J P  D   NSg/I/J R      V/J  NSg/V/C D   NSg/J . NSg/J       NSg+
> algorithm .
# NSg+      .
>
#
> Dynamic programming methods
# NSg/J+  NSg/V+      NPl/V
>
#
> In        1987 , Steven DeRose and Kenneth W. Church   independently developed dynamic
# NPrSg/J/P #    . NPr+   ?      V/C NPr+    ?  NPrSg/V+ R             V/J       NSg/J
> programming algorithms to solve the same problem in        vastly less    time     . Their
# NSg/V+      NPl+       P  NSg/V D   I/J  NSg/J   NPrSg/J/P R      V/J/C/P NSg/V/J+ . D$+
> methods were  similar to the Viterbi algorithm known   for some  time     in        other
# NPl/V+  NSg/V NSg/J   P  D   ?       NSg       NSg/V/J C/P I/J/R NSg/V/J+ NPrSg/J/P NSg/V/J+
> fields   . DeRose used a   table of pairs  , while     Church   used a   table of triples and a
# NPrPl/V+ . ?      V/J  D/P NSg/V P  NPl/V+ . NSg/V/C/P NPrSg/V+ V/J  D/P NSg/V P  NPl/V   V/C D/P
> method of estimating the values for triples that        were  rare    or      nonexistent in        the
# NSg/V  P  V          D   NPl/V  C/P NPl/V   N/I/C/Ddem+ NSg/V NSg/V/J NPrSg/C NSg/J       NPrSg/J/P D+
> Brown      Corpus ( an  actual measurement of triple  probabilities would  require a   much
# NPrSg/V/J+ NSg    . D/P NSg/J  NSg         P  NSg/V/J NPl+          NSg/VX NSg/V   D/P N/I/J
> larger corpus ) . Both methods achieved an  accuracy of over      95 % . DeRose's 1990
# J      NSg+   . . I/C  NPl/V+  V/J      D/P NSg      P  NSg/V/J/P #  . . ?        #
> dissertation at    Brown     University included analyses    of the specific error  types  ,
# NSg+         NSg/P NPrSg/V/J NSg+       V/J      NSg/V/Au/Br P  D+  NSg/J+   NSg/V+ NPl/V+ .
> probabilities , and other    related data , and replicated his     work  for Greek     , where
# NPl+          . V/C NSg/V/J+ J+      NSg+ . V/C V/J        ISg/D$+ NSg/V C/P NPrSg/V/J . NSg/C
> it         proved similarly effective .
# NPrSg/ISg+ V/J    R+        NSg/J     .
>
#
> These   findings were  surprisingly disruptive to the field of natural language
# I/Ddem+ NSg      NSg/V R            J          P  D   NSg/V P  NSg/J+  NSg/V+
> processing . The accuracy reported was higher than the typical accuracy of very
# V+         . D+  NSg+     V/J      V   J      C/P  D   NSg/J   NSg      P  J
> sophisticated algorithms that        integrated part    of speech choice with many    higher
# V/J           NPl+       N/I/C/Ddem+ V/J        NSg/V/J P  NSg/V+ NSg/J  P    N/I/J/D J
> levels of linguistic analysis : syntax , morphology , semantics , and so         on  . CLAWS ,
# NPl/V  P  J          NSg+     . NSg+   . NSg+       . NSg+      . V/C NSg/I/J/C+ J/P . NPl/V .
> DeRose's and Church's methods did fail    for some  of the known    cases  where
# ?        V/C N$       NPl/V+  V   NSg/V/J C/P I/J/R P  D+  NSg/V/J+ NPl/V+ NSg/C
> semantics is required , but     those   proved negligibly rare     . This    convinced many    in
# NSg+      VL V/J      . NSg/C/P I/Ddem+ V/J    R+         NSg/V/J+ . I/Ddem+ V/J       N/I/J/D NPrSg/J/P
> the field  that        part    - of - speech tagging could  usefully be     separated from the other
# D+  NSg/V+ N/I/C/Ddem+ NSg/V/J . P  . NSg/V  NSg/V   NSg/VX R        NSg/VX V/J       P    D   NSg/V/J
> levels of processing ; this    , in        turn  , simplified the theory and practice of
# NPl/V  P  V          . I/Ddem+ . NPrSg/J/P NSg/V . V/J        D+  NSg    V/C NSg/V    P
> computerized language analysis and encouraged researchers to find  ways to
# V/J          NSg/V+   NSg+     V/C V/J        +           P  NSg/V NPl+ P
> separate other    pieces as     well    . Markov Models became the standard method for the
# NSg/V/J  NSg/V/J+ NPl/V+ NSg/R+ NSg/V/J . NPr    NPl/V+ V      D   NSg/J    NSg/V  C/P D
> part    - of - speech assignment .
# NSg/V/J . P  . NSg/V+ NSg+       .
>
#
> Unsupervised taggers
# V/J+         NPl
>
#
> The methods already discussed involve working from a   pre      - existing corpus to
# D+  NPl/V   W?      V/J       V       V       P    D/P NSg/V/P+ . V        NSg    P
> learn tag    probabilities . It         is , however , also possible to bootstrap using
# NSg/V NSg/V+ NPl+          . NPrSg/ISg+ VL . C       . W?   NSg/J    P  NSg/V     V
> " unsupervised " tagging . Unsupervised tagging techniques use   an  untagged corpus
# . V/J          . NSg/V   . V/J          NSg/V   NPl+       NSg/V D/P ?        NSg
> for their training data and produce the tagset by       induction . That        is , they
# C/P D$+   NSg/V+   NSg+ V/C NSg/V   D   NSg    NSg/J/P+ NSg       . N/I/C/Ddem+ VL . IPl+
> observe patterns in        word   use   , and derive part    - of - speech categories themselves .
# NSg/V   NPl/V+   NPrSg/J/P NSg/V+ NSg/V . V/C NSg/V  NSg/V/J . P  . NSg/V  NPl+       I+         .
> For example , statistics readily reveal that        " the " , " a   " , and " an  " occur in
# C/P NSg/V+  . NPl/V+     R       NSg/V  N/I/C/Ddem+ . D   . . . D/P . . V/C . D/P . V     NPrSg/J/P
> similar contexts , while     " eat   " occurs in        very different ones   . With sufficient
# NSg/J+  NPl/V+   . NSg/V/C/P . NSg/V . V      NPrSg/J/P J    NSg/J+    NPl/V+ . P    J+
> iteration , similarity classes of words  emerge that        are remarkably similar to
# NSg       . NSg        NPl/V   P  NPl/V+ NSg/V  N/I/C/Ddem+ V   R          NSg/J   P
> those   human   linguists would  expect ; and the differences themselves sometimes
# I/Ddem+ NSg/V/J NPl+      NSg/VX V      . V/C D+  NSg/V+      I+         R
> suggest valuable new      insights .
# V       NSg/J+   NSg/V/J+ NPl+     .
>
#
> These  two  categories can      be     further subdivided into rule  - based , stochastic , and
# I/Ddem NSg+ NPl        NPrSg/VX NSg/VX V/J     V/J        P    NSg/V . V/J   . J          . V/C
> neural approaches .
# J+     NPl/V+     .
>
#
> Other    taggers and methods
# NSg/V/J+ NPl     V/C NPl/V
>
#
> Some   current major     algorithms for part    - of - speech tagging include the Viterbi
# I/J/R+ NSg/J   NPrSg/V/J NPl        C/P NSg/V/J . P  . NSg/V  NSg/V   NSg/V   D   ?
> algorithm , Brill tagger , Constraint Grammar , and the Baum - Welch algorithm ( also
# NSg       . NSg/J NSg    . NSg+       NSg/V+  . V/C D   NPr  . ?     NSg       . W?
> known   as    the forward - backward algorithm ) . Hidden Markov model    and visible Markov
# NSg/V/J NSg/R D   NSg/V/J . NSg/J    NSg+      . . V/J    NPr    NSg/V/J+ V/C J       NPr
> model    taggers can      both be     implemented using the Viterbi algorithm . The
# NSg/V/J+ NPl     NPrSg/VX I/C  NSg/VX V/J         V     D+  ?       NSg       . D
> rule   - based Brill tagger is unusual in        that       it         learns a   set       of rule   patterns , and
# NSg/V+ . V/J   NSg/J NSg    VL NSg/J   NPrSg/J/P N/I/C/Ddem NPrSg/ISg+ NPl/V  D/P NPrSg/V/J P  NSg/V+ NPl/V+   . V/C
> then    applies those   patterns rather    than optimizing a    statistical quantity .
# NSg/J/C V       I/Ddem+ NPl/V+   NPrSg/V/J C/P  V          D/P+ J+          NSg+     .
>
#
> Many     machine learning methods have   also been  applied to the problem of POS
# N/I/J/D+ NSg/V   V+       NPl/V+  NSg/VX W?   NSg/V V/J     P  D   NSg/J   P  NSg+
> tagging . Methods such  as    SVM , maximum entropy classifier , perceptron , and
# NSg/V+  . NPl/V+  NSg/I NSg/R ?   . NSg/J   NSg     NSg        . N          . V/C
> nearest - neighbor have   all       been  tried , and most    can      achieve accuracy above
# W?      . NSg/V/J  NSg/VX NSg/I/J/C NSg/V V/J   . V/C NSg/I/J NPrSg/VX V       NSg+     NSg/J/P
> 95 % . [ citation needed ]
# #  . . . NSg+     V/J+   .
>
#
> A   direct comparison of several methods is reported ( with references ) at    the ACL
# D/P V/J    NSg        P  J/D+    NPl/V+  VL V/J      . P    NPl/V+     . NSg/P D+  NSg+
> Wiki   . This    comparison uses  the Penn tag    set       on  some  of the Penn Treebank data ,
# NSg/V+ . I/Ddem+ NSg+       NPl/V D+  NPr+ NSg/V+ NPrSg/V/J J/P I/J/R P  D+  NPr+ ?        NSg+ .
> so        the results are directly comparable . However , many    significant taggers are
# NSg/I/J/C D+  NPl/V+  V   R/C      NSg/J+     . C       . N/I/J/D NSg/J       NPl     V
> not   included ( perhaps because of the labor          involved in        reconfiguring them for
# NSg/C V/J      . NSg     C/P     P  D+  NPrSg/V/Am/Au+ V/J      NPrSg/J/P V             N/I+ C/P
> this    particular dataset ) . Thus , it         should not   be     assumed that       the results
# I/Ddem+ NSg/J+     NSg     . . NSg  . NPrSg/ISg+ VX     NSg/C NSg/VX V/J     N/I/C/Ddem D+  NPl/V+
> reported here    are the best       that        can      be     achieved with a    given      approach ; nor   even
# V/J      NSg/J/R V   D   NPrSg/VX/J N/I/C/Ddem+ NPrSg/VX NSg/VX V/J      P    D/P+ NSg/V/J/P+ NSg/V+   . NSg/C NSg/V/J
> the best        that        have   been  achieved with a    given      approach .
# D+  NPrSg/VX/J+ N/I/C/Ddem+ NSg/VX NSg/V V/J      P    D/P+ NSg/V/J/P+ NSg/V+   .
>
#
> In        2014 , a    paper    reporting using the structure regularization method for
# NPrSg/J/P #    . D/P+ NSg/V/J+ V         V     D+  NSg/V+    NSg            NSg/V  C/P
> part    - of - speech tagging , achieving 97.36 % on  a   standard benchmark dataset .
# NSg/V/J . P  . NSg/V  NSg/V   . V         #     . J/P D/P NSg/J+   NSg/V+    NSg     .
