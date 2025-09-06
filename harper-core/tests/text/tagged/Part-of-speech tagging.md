> <!--
# Unlintable
>            source: https://en.wikipedia.org/w/index.php?title=Part-of-speech_tagging&oldid=1275774341
# Unlintable Unlintable
>            license: CC BY-SA 4.0
# Unlintable Unlintable
>            -->
# Unlintable Unlintable
>            Part      - of - speech   tagging
# Unlintable NSg/VB/J+ . P  . N🅪Sg/VB+ NSg/Vg
>
#
> In      corpus linguistics , part      - of - speech   tagging ( POS  tagging or    PoS  tagging or
# NPr/J/P NSg+   Nᴹ+         . NSg/VB/J+ . P  . N🅪Sg/VB+ NSg/Vg  . NSg+ NSg/Vg  NPr/C NSg+ NSg/Vg  NPr/C
> POST       ) , also called grammatical tagging is  the process of marking up         a   word    in      a
# NPr🅪/VB/P+ . . R/C  VP/J   J           NSg/Vg  VL3 D   NSg/VB  P  Nᴹ/Vg/J NSg/VB/J/P D/P NSg/VB+ NPr/J/P D/P
> text     ( corpus ) as    corresponding to a   particular part     of speech   , based on  both   its
# N🅪Sg/VB+ . NSg+   . NSg/R Nᴹ/Vg/J       P  D/P NSg/J      NSg/VB/J P  N🅪Sg/VB+ . VP/J  J/P I/C/Dq ISg/D$+
> definition and  its     context  . A   simplified form    of this    is  commonly taught to
# NSg        VB/C ISg/D$+ N🅪Sg/VB+ . D/P VP/J       N🅪Sg/VB P  I/Ddem+ VL3 R        VB     P
> school  - age      children , in      the identification of words   as    nouns  , verbs   , adjectives ,
# N🅪Sg/VB . N🅪Sg/VB+ NPl+     . NPr/J/P D   Nᴹ             P  NPl/V3+ NSg/R NPl/V3 . NPl/V3+ . NPl/V3     .
> adverbs , etc.
# NPl/V3  . +
>
#
> Once  performed by      hand    , POS  tagging is  now        done      in      the context of computational
# NSg/C VP/J      NSg/J/P NSg/VB+ . NSg+ NSg/Vg  VL3 NPr/VB/J/C NSg/VPp/J NPr/J/P D   N🅪Sg/VB P  J
> linguistics , using   algorithms which associate discrete terms   , as    well     as    hidden
# Nᴹ+         . Nᴹ/Vg/J NPl+       I/C+  NSg/VB/J+ J        NPl/V3+ . NSg/R NSg/VB/J NSg/R VB/J
> parts  of speech   , by      a   set      of descriptive tags    . POS  - tagging algorithms fall     into
# NPl/V3 P  N🅪Sg/VB+ . NSg/J/P D/P NPr/VB/J P  NSg/J       NPl/V3+ . NSg+ . NSg/Vg  NPl+       N🅪Sg/VB+ P
> two distinctive groups  : rule    - based and  stochastic . E. Brill's tagger , one        of the
# NSg NSg/J       NPl/V3+ . NSg/VB+ . VP/J  VB/C J          . ?  ?       NSg    . NSg/I/VB/J P  D
> first    and  most         widely used English    POS  - taggers , employs rule    - based algorithms .
# NSg/VB/J VB/C NSg/I/J/R/Dq R      VP/J NPr🅪/VB/J+ NSg+ . NPl     . NPl/V3  NSg/VB+ . VP/J  NPl+       .
>
#
> Principle
# N🅪Sg/VB+
>
#
> Part      - of - speech   tagging is  harder than just having  a   list   of words   and  their
# NSg/VB/J+ . P  . N🅪Sg/VB+ NSg/Vg  VL3 JC     C/P  VB/J Nᴹ/Vg/J D/P NSg/VB P  NPl/V3+ VB/C D$+
> parts  of speech   , because some     words   can     represent more            than one        part     of speech
# NPl/V3 P  N🅪Sg/VB+ . C/P     I/J/R/Dq NPl/V3+ NPr/VXB VB        NPr/I/VB/J/R/Dq C/P  NSg/I/VB/J NSg/VB/J P  N🅪Sg/VB+
> at    different times   , and  because some     parts  of speech   are complex  . This    is  not
# NSg/P NSg/J     NPl/V3+ . VB/C C/P     I/J/R/Dq NPl/V3 P  N🅪Sg/VB+ VB  NSg/VB/J . I/Ddem+ VL3 NSg/C
> rare     — in      natural languages ( as    opposed to many        artificial languages ) , a   large
# NSg/VB/J . NPr/J/P NSg/J+  NPl/V3+   . NSg/R VP/J    P  NSg/I/J/Dq+ J+         NPl/V3+   . . D/P NSg/J
> percentage of word    - forms   are ambiguous . For example , even     " dogs    " , which is
# N🅪Sg       P  NSg/VB+ . NPl/V3+ VB  J         . C/P NSg/VB+ . NSg/VB/J . NPl/V3+ . . I/C+  VL3
> usually thought of as    just a    plural noun    , can     also be      a    verb    :
# R       N🅪Sg/VP P  NSg/R VB/J D/P+ NSg/J+ NSg/VB+ . NPr/VXB R/C  NSg/VXB D/P+ NSg/VB+ .
>
#
> The sailor dogs    the hatch   .
# D+  NSg+   NPl/V3+ D+  NSg/VB+ .
>
#
> Correct  grammatical tagging will    reflect that          " dogs    " is  here    used as    a   verb    , not
# NSg/VB/J J           NSg/Vg  NPr/VXB VB      NSg/I/C/Ddem+ . NPl/V3+ . VL3 NSg/J/R VP/J NSg/R D/P NSg/VB+ . NSg/C
> as    the more            common   plural noun    . Grammatical context  is  one        way   to determine
# NSg/R D   NPr/I/VB/J/R/Dq NSg/VB/J NSg/J  NSg/VB+ . J+          N🅪Sg/VB+ VL3 NSg/I/VB/J NSg/J P  VB
> this    ; semantic analysis can     also be      used to infer that          " sailor " and  " hatch  "
# I/Ddem+ . NSg/J+   N🅪Sg+    NPr/VXB R/C  NSg/VXB VP/J P  VB    NSg/I/C/Ddem+ . NSg+   . VB/C . NSg/VB .
> implicate " dogs    " as    1 ) in      the nautical context  and  2 ) an  action     applied to the
# NSg/VB    . NPl/V3+ . NSg/R # . NPr/J/P D   J        N🅪Sg/VB+ VB/C # . D/P N🅪Sg/VB/J+ VP/J    P  D
> object  " hatch  " ( in      this   context  , " dogs    " is  a   nautical term      meaning    " fastens ( a
# NSg/VB+ . NSg/VB . . NPr/J/P I/Ddem N🅪Sg/VB+ . . NPl/V3+ . VL3 D/P J        NSg/VB/J+ N🅪Sg/Vg/J+ . V3      . D/P
> watertight door    ) securely " ) .
# J          NSg/VB+ . R        . . .
>
#
> Tag     sets
# NSg/VB+ NPl/V3
>
#
> Schools commonly teach  that         there are 9 parts  of speech  in      English   : noun    , verb    ,
# NPl/V3+ R        NSg/VB NSg/I/C/Ddem +     VB  # NPl/V3 P  N🅪Sg/VB NPr/J/P NPr🅪/VB/J . NSg/VB+ . NSg/VB+ .
> article , adjective , preposition , pronoun , adverb  , conjunction , and  interjection .
# NSg/VB+ . NSg/VB/J+ . NSg/VB      . NSg/VB+ . NSg/VB+ . NSg/VB+     . VB/C N🅪Sg+        .
> However , there are clearly many        more             categories and  sub      - categories . For nouns  ,
# C       . +     VB  R       NSg/I/J/Dq+ NPr/I/VB/J/R/Dq+ NPl+       VB/C NSg/VB/P . NPl+       . C/P NPl/V3 .
> the plural , possessive , and  singular forms   can     be      distinguished . In      many
# D   NSg/J  . NSg/J      . VB/C NSg/J    NPl/V3+ NPr/VXB NSg/VXB VP/J          . NPr/J/P NSg/I/J/Dq+
> languages words   are also marked for their " case     " ( role as    subject   , object  ,
# NPl/V3+   NPl/V3+ VB  R/C  VP/J   C/P D$+   . NPr🅪/VB+ . . NSg  NSg/R NSg/VB/J+ . NSg/VB+ .
> etc. ) , grammatical gender     , and  so        on  ; while      verbs   are marked for tense    , aspect  ,
# +    . . J+          N🅪Sg/VB/J+ . VB/C NSg/I/J/C J/P . NSg/VB/C/P NPl/V3+ VB  VP/J   C/P NSg/VB/J . NSg/VB+ .
> and  other     things . In      some     tagging systems , different inflections of the same
# VB/C NSg/VB/J+ NPl+   . NPr/J/P I/J/R/Dq NSg/Vg  NPl+    . NSg/J     NPl         P  D   I/J
> root    word    will    get    different parts  of speech   , resulting in      a   large number     of
# NPr/VB+ NSg/VB+ NPr/VXB NSg/VB NSg/J     NPl/V3 P  N🅪Sg/VB+ . Nᴹ/Vg/J   NPr/J/P D/P NSg/J N🅪Sg/VB/JC P
> tags    . For example , NN for singular common   nouns  , NNS for plural common   nouns  , NP
# NPl/V3+ . C/P NSg/VB+ . ?  C/P NSg/J    NSg/VB/J NPl/V3 . ?   C/P NSg/J  NSg/VB/J NPl/V3 . NPr
> for singular proper nouns  ( see    the POS  tags    used in      the Brown     Corpus ) . Other
# C/P NSg/J    NSg/J  NPl/V3 . NSg/VB D   NSg+ NPl/V3+ VP/J NPr/J/P D   NPr🅪/VB/J NSg+   . . NSg/VB/J
> tagging systems use     a   smaller number     of tags    and  ignore fine     differences or
# NSg/Vg  NPl+    N🅪Sg/VB D/P NSg/JC  N🅪Sg/VB/JC P  NPl/V3+ VB/C VB     NSg/VB/J NPl/VB      NPr/C
> model     them     as    features somewhat independent from part      - of - speech   .
# NSg/VB/J+ NSg/IPl+ NSg/R NPl/V3+  NSg/I/R  NSg/J       P    NSg/VB/J+ . P  . N🅪Sg/VB+ .
>
#
> In      part      - of - speech   tagging by      computer , it       is  typical to distinguish from 50 to
# NPr/J/P NSg/VB/J+ . P  . N🅪Sg/VB+ NSg/Vg  NSg/J/P NSg/VB+  . NPr/ISg+ VL3 NSg/J   P  VB          P    #  P
> 150 separate parts  of speech  for English    . Work    on  stochastic methods for tagging
# #   NSg/VB/J NPl/V3 P  N🅪Sg/VB C/P NPr🅪/VB/J+ . N🅪Sg/VB J/P J          NPl/V3+ C/P NSg/Vg
> Koine Greek    ( DeRose 1990 ) has used over    1 , 000 parts  of speech   and  found  that
# ?     NPr/VB/J . ?      #    . V3  VP/J NSg/J/P # . #   NPl/V3 P  N🅪Sg/VB+ VB/C NSg/VB NSg/I/C/Ddem
> about as    many       words   were    ambiguous in      that         language as    in      English    . A
# J/P   NSg/R NSg/I/J/Dq NPl/V3+ NSg/VPt J         NPr/J/P NSg/I/C/Ddem N🅪Sg/VB+ NSg/R NPr/J/P NPr🅪/VB/J+ . D/P
> morphosyntactic descriptor in      the case    of morphologically rich     languages is
# ?               NSg        NPr/J/P D   NPr🅪/VB P  ?               NPr/VB/J NPl/V3+   VL3
> commonly expressed using   very short      mnemonics , such  as    Ncmsan for Category = Noun    ,
# R        VP/J      Nᴹ/Vg/J J/R  NPr/VB/J/P NPl       . NSg/I NSg/R ?      C/P NSg+     . NSg/VB+ .
> Type    = common   , Gender     = masculine , Number      = singular , Case     = accusative , Animate
# NSg/VB+ . NSg/VB/J . N🅪Sg/VB/J+ . NSg/J     . N🅪Sg/VB/JC+ . NSg/J    . NPr🅪/VB+ . NSg/J      . VB/J
> = no    .
# . NPr/P .
>
#
> The most         popular " tag    set      " for POS  tagging for American English    is  probably the
# D   NSg/I/J/R/Dq NSg/J   . NSg/VB NPr/VB/J . C/P NSg+ NSg/Vg  C/P NPr/J    NPr🅪/VB/J+ VL3 R        D
> Penn tag     set      , developed in      the Penn Treebank project . It       is  largely similar to
# NPr+ NSg/VB+ NPr/VB/J . VP/J      NPr/J/P D   NPr+ ?        NSg/VB+ . NPr/ISg+ VL3 R       NSg/J   P
> the earlier Brown     Corpus and  LOB    Corpus tag     sets   , though much         smaller . In
# D   JC      NPr🅪/VB/J NSg    VB/C NSg/VB NSg+   NSg/VB+ NPl/V3 . VB/C   NSg/I/J/R/Dq NSg/JC  . NPr/J/P
> Europe , tag     sets   from the Eagles Guidelines see    wide  use      and  include versions
# NPr+   . NSg/VB+ NPl/V3 P    D   NPl/V3 NPl+       NSg/VB NSg/J N🅪Sg/VB+ VB/C NSg/VB  NPl/V3+
> for multiple languages .
# C/P NSg/J/Dq NPl/V3+   .
>
#
> POS  tagging work     has been    done      in      a   variety of languages , and  the set      of POS
# NSg+ NSg/Vg  N🅪Sg/VB+ V3  NSg/VPp NSg/VPp/J NPr/J/P D/P NSg     P  NPl/V3+   . VB/C D   NPr/VB/J P  NSg+
> tags    used varies greatly with language . Tags    usually are designed to include
# NPl/V3+ VP/J NPl/V3 R       P    N🅪Sg/VB+ . NPl/V3+ R       VB  VP/J     P  NSg/VB
> overt  morphological distinctions , although this   leads  to inconsistencies such  as
# NSg/J+ J+            NPl+         . C        I/Ddem NPl/V3 P  NPl             NSg/I NSg/R
> case     - marking for pronouns but     not   nouns  in      English    , and  much         larger
# NPr🅪/VB+ . Nᴹ/Vg/J C/P NPl/V3   NSg/C/P NSg/C NPl/V3 NPr/J/P NPr🅪/VB/J+ . VB/C NSg/I/J/R/Dq JC
> cross       - language differences . The tag     sets   for heavily inflected languages such  as
# NPr/VB/J/P+ . N🅪Sg/VB+ NPl/VB+     . D+  NSg/VB+ NPl/V3 C/P R       VP/J      NPl/V3+   NSg/I NSg/R
> Greek    and  Latin can     be      very large ; tagging words   in      agglutinative languages such
# NPr/VB/J VB/C NPr/J NPr/VXB NSg/VXB J/R  NSg/J . NSg/Vg  NPl/V3+ NPr/J/P ?             NPl/V3+   NSg/I
> as    Inuit languages may     be      virtually impossible . At    the other    extreme , Petrov et
# NSg/R NPr/J NPl/V3+   NPr/VXB NSg/VXB R         NSg/J      . NSg/P D   NSg/VB/J NSg/J   . ?      ?
> al. have    proposed a   " universal " tag     set      , with 12 categories ( for example , no
# ?   NSg/VXB VP/J     D/P . NSg/J     . NSg/VB+ NPr/VB/J . P    #  NPl+       . C/P NSg/VB+ . NPr/P
> subtypes of nouns  , verbs   , punctuation , and  so        on  ) . Whether a   very small    set      of
# NPl      P  NPl/V3 . NPl/V3+ . Nᴹ+         . VB/C NSg/I/J/C J/P . . I/C     D/P J/R  NPr/VB/J NPr/VB/J P
> very broad tags    or    a   much         larger set      of more            precise ones    is  preferable , depends
# J/R  NSg/J NPl/V3+ NPr/C D/P NSg/I/J/R/Dq JC     NPr/VB/J P  NPr/I/VB/J/R/Dq VB/J+   NPl/V3+ VL3 W?         . NPl/V3
> on  the purpose  at    hand    . Automatic tagging is  easier on  smaller tag     - sets   .
# J/P D   N🅪Sg/VB+ NSg/P NSg/VB+ . NSg/J     NSg/Vg  VL3 NSg/JC J/P NSg/JC  NSg/VB+ . NPl/V3 .
>
#
> History
# N🅪Sg+
>
#
> The Brown      Corpus
# D+  NPr🅪/VB/J+ NSg+
>
#
> Research on  part      - of - speech   tagging has been    closely tied to corpus linguistics .
# Nᴹ/VB    J/P NSg/VB/J+ . P  . N🅪Sg/VB+ NSg/Vg  V3  NSg/VPp R       VP/J P  NSg    Nᴹ+         .
> The first    major    corpus of English   for computer analysis was the Brown     Corpus
# D   NSg/VB/J NPr/VB/J NSg    P  NPr🅪/VB/J C/P NSg/VB+  N🅪Sg+    VPt D   NPr🅪/VB/J NSg
> developed at    Brown     University by      Henry Kučera and  W. Nelson Francis , in      the
# VP/J      NSg/P NPr🅪/VB/J NSg+       NSg/J/P NPr+  ?      VB/C ?  NPr+   NPr+    . NPr/J/P D
> mid      - 1960s . It       consists of about 1 , 000 , 000 words  of running   English    prose text     ,
# NSg/J/P+ . #d    . NPr/ISg+ NPl/V3   P  J/P   # . #   . #   NPl/V3 P  Nᴹ/Vg/J/P NPr🅪/VB/J+ Nᴹ/VB N🅪Sg/VB+ .
> made up         of 500 samples from randomly chosen   publications . Each sample  is  2 , 000
# VB   NSg/VB/J/P P  #   NPl/V3+ P    R        Nᴹ/VPp/J NPl+         . Dq+  NSg/VB+ VL3 # . #
> or    more            words   ( ending  at    the first    sentence - end     after 2 , 000 words   , so        that         the
# NPr/C NPr/I/VB/J/R/Dq NPl/V3+ . Nᴹ/Vg/J NSg/P D   NSg/VB/J NSg/VB+  . NSg/VB+ P     # . #   NPl/V3+ . NSg/I/J/C NSg/I/C/Ddem D
> corpus contains only  complete sentences ) .
# NSg+   V3       J/R/C NSg/VB/J NPl/V3+   . .
>
#
> The Brown      Corpus was painstakingly " tagged " with part      - of - speech   markers over
# D+  NPr🅪/VB/J+ NSg+   VPt R             . VP/J   . P    NSg/VB/J+ . P  . N🅪Sg/VB+ NPl/V3  NSg/J/P
> many        years . A    first     approximation was done      with a    program by      Greene and  Rubin ,
# NSg/I/J/Dq+ NPl+  . D/P+ NSg/VB/J+ N🅪Sg+         VPt NSg/VPp/J P    D/P+ NPr/VB+ NSg/J/P NPr    VB/C NPr   .
> which consisted of a   huge handmade list   of what   categories could   co        - occur at
# I/C+  VP/J      P  D/P J    NSg/J    NSg/VB P  NSg/I+ NPl+       NSg/VXB NPr/I/VB+ . VB    NSg/P
> all          . For example , article then    noun    can     occur , but     article then    verb    ( arguably )
# NSg/I/J/C/Dq . C/P NSg/VB+ . NSg/VB+ NSg/J/C NSg/VB+ NPr/VXB VB    . NSg/C/P NSg/VB+ NSg/J/C NSg/VB+ . R        .
> cannot . The program got about 70 % correct  . Its     results were    repeatedly reviewed
# NSg/VB . D+  NPr/VB+ VB  J/P   #  . NSg/VB/J . ISg/D$+ NPl/V3+ NSg/VPt R          VP/J
> and  corrected by      hand    , and  later users sent   in      errata so        that          by      the late  70 s
# VB/C VP/J      NSg/J/P NSg/VB+ . VB/C JC    NPl+  NSg/VB NPr/J/P NSg    NSg/I/J/C NSg/I/C/Ddem+ NSg/J/P D   NSg/J #  ?
> the tagging was nearly perfect  ( allowing for some     cases   on  which even     human
# D   NSg/Vg  VPt R      NSg/VB/J . Nᴹ/Vg/J  C/P I/J/R/Dq NPl/V3+ J/P I/C+  NSg/VB/J NSg/VB/J+
> speakers might    not   agree ) .
# +        Nᴹ/VXB/J NSg/C VB    . .
>
#
> This    corpus has been    used for innumerable studies of word    - frequency and  of
# I/Ddem+ NSg+   V3  NSg/VPp VP/J C/P J           NPl/V3  P  NSg/VB+ . NSg       VB/C P
> part      - of - speech   and  inspired the development of similar " tagged " corpora in      many
# NSg/VB/J+ . P  . N🅪Sg/VB+ VB/C VP/J     D   N🅪Sg        P  NSg/J   . VP/J   . NPl+    NPr/J/P NSg/I/J/Dq
> other    languages . Statistics derived by      analyzing it       formed the basis for most
# NSg/VB/J NPl/V3+   . NPl/V3+    VP/J    NSg/J/P Nᴹ/Vg/J   NPr/ISg+ VP/J   D+  NSg+  C/P NSg/I/J/R/Dq
> later part      - of - speech   tagging systems , such  as    CLAWS   and  VOLSUNGA . However , by
# JC    NSg/VB/J+ . P  . N🅪Sg/VB+ NSg/Vg  NPl+    . NSg/I NSg/R NPl/V3+ VB/C ?        . C       . NSg/J/P
> this    time       ( 2005 ) it       has been    superseded by      larger corpora such  as    the 100
# I/Ddem+ N🅪Sg/VB/J+ . #    . NPr/ISg+ V3  NSg/VPp VP/J       NSg/J/P JC     NPl+    NSg/I NSg/R D   #
> million word    British National Corpus , even     though larger corpora are rarely so
# NSg     NSg/VB+ NPr/J   NSg/J    NSg+   . NSg/VB/J VB/C   JC     NPl+    VB  R      NSg/I/J/C
> thoroughly curated .
# R          VP/J    .
>
#
> For some     time       , part      - of - speech   tagging was considered an  inseparable part     of
# C/P I/J/R/Dq N🅪Sg/VB/J+ . NSg/VB/J+ . P  . N🅪Sg/VB+ NSg/Vg  VPt VP/J       D/P NSg/J       NSg/VB/J P
> natural language processing , because there are certain cases   where the correct
# NSg/J   N🅪Sg/VB+ Nᴹ/Vg/J+   . C/P     +     VB  I/J     NPl/V3+ NSg/C D   NSg/VB/J
> part     of speech   cannot be      decided  without understanding the semantics or    even     the
# NSg/VB/J P  N🅪Sg/VB+ NSg/VB NSg/VXB NSg/VP/J C/P     N🅪Sg/Vg/J+    D   NPl+      NPr/C NSg/VB/J D
> pragmatics of the context  . This    is  extremely expensive , especially because
# NPl        P  D   N🅪Sg/VB+ . I/Ddem+ VL3 R         J         . R          C/P
> analyzing the higher  levels  is  much         harder when    multiple part     - of - speech
# Nᴹ/Vg/J   D+  NSg/JC+ NPl/V3+ VL3 NSg/I/J/R/Dq JC     NSg/I/C NSg/J/Dq NSg/VB/J . P  . N🅪Sg/VB+
> possibilities must   be      considered for each word    .
# NPl+          NSg/VB NSg/VXB VP/J       C/P Dq+  NSg/VB+ .
>
#
> Use     of hidden Markov models
# N🅪Sg/VB P  VB/J   NPr    NPl/V3+
>
#
> In      the mid      - 1980s , researchers in      Europe began to use     hidden Markov models  ( HMMs )
# NPr/J/P D   NSg/J/P+ . #d    . NPl         NPr/J/P NPr+   VPt   P  N🅪Sg/VB VB/J   NPr    NPl/V3+ . ?    .
> to disambiguate parts  of speech   , when    working to tag    the Lancaster - Oslo - Bergen
# P  VB           NPl/V3 P  N🅪Sg/VB+ . NSg/I/C Nᴹ/Vg/J P  NSg/VB D   NPr       . NPr+ . NPr+
> Corpus of British English    . HMMs involve counting cases   ( such  as    from the Brown
# NSg    P  NPr/J   NPr🅪/VB/J+ . ?    VB      Nᴹ/Vg/J  NPl/V3+ . NSg/I NSg/R P    D   NPr🅪/VB/J
> Corpus ) and  making  a   table  of the probabilities of certain sequences . For
# NSg+   . VB/C Nᴹ/Vg/J D/P NSg/VB P  D   NPl           P  I/J     NPl/V3+   . C/P
> example , once  you've seen    an  article such  as    ' the ' , perhaps the next    word    is  a
# NSg/VB+ . NSg/C W?     NSg/VPp D/P NSg/VB+ NSg/I NSg/R . D   . . NSg/R   D   NSg/J/P NSg/VB+ VL3 D/P
> noun    40 % of the time       , an  adjective 40 % , and  a   number      20 % . Knowing    this    , a
# NSg/VB+ #  . P  D   N🅪Sg/VB/J+ . D/P NSg/VB/J+ #  . . VB/C D/P N🅪Sg/VB/JC+ #  . . NSg/VB/J/P I/Ddem+ . D/P+
> program can     decide that          " can     " in      " the can     " is  far      more            likely to be      a   noun   than
# NPr/VB+ NPr/VXB VB     NSg/I/C/Ddem+ . NPr/VXB . NPr/J/P . D+  NPr/VXB . VL3 NSg/VB/J NPr/I/VB/J/R/Dq NSg/J  P  NSg/VXB D/P NSg/VB C/P
> a    verb    or    a   modal . The same method  can     , of course  , be      used to benefit from
# D/P+ NSg/VB+ NPr/C D/P NSg/J . D+  I/J+ NSg/VB+ NPr/VXB . P  NSg/VB+ . NSg/VXB VP/J P  NSg/VB  P
> knowledge about the following words   .
# Nᴹ+       J/P   D+  Nᴹ/Vg/J/P NPl/V3+ .
>
#
> More            advanced ( " higher - order  " ) HMMs learn  the probabilities not   only  of pairs
# NPr/I/VB/J/R/Dq VP/J     . . NSg/JC . NSg/VB . . ?    NSg/VB D   NPl+          NSg/C J/R/C P  NPl/V3+
> but     triples or    even     larger sequences . So        , for example , if    you've just seen    a
# NSg/C/P NPl/V3  NPr/C NSg/VB/J JC     NPl/V3+   . NSg/I/J/C . C/P NSg/VB+ . NSg/C W?     VB/J NSg/VPp D/P
> noun    followed by      a   verb    , the next    item    may     be      very likely a   preposition ,
# NSg/VB+ VP/J     NSg/J/P D/P NSg/VB+ . D   NSg/J/P NSg/VB+ NPr/VXB NSg/VXB J/R  NSg/J  D/P NSg/VB      .
> article , or    noun    , but     much         less       likely another verb    .
# NSg/VB+ . NPr/C NSg/VB+ . NSg/C/P NSg/I/J/R/Dq VB/J/R/C/P NSg/J  I/D     NSg/VB+ .
>
#
> When    several ambiguous words   occur together , the possibilities multiply .
# NSg/I/C J/Dq+   J+        NPl/V3+ VB    J        . D+  NPl+          NSg/VB   .
> However , it       is  easy     to enumerate every combination and  to assign a   relative
# C       . NPr/ISg+ VL3 NSg/VB/J P  VB        Dq+   N🅪Sg+       VB/C P  NSg/VB D/P NSg/J
> probability to each one         , by      multiplying together the probabilities of each
# NSg+        P  Dq   NSg/I/VB/J+ . NSg/J/P Nᴹ/Vg/J     J        D   NPl           P  Dq
> choice  in      turn   . The combination with the highest probability is  then    chosen   . The
# N🅪Sg/J+ NPr/J/P NSg/VB . D   N🅪Sg        P    D+  JS+     NSg+        VL3 NSg/J/C Nᴹ/VPp/J . D+
> European group   developed CLAWS   , a   tagging program that          did exactly this   and
# NSg/J+   NSg/VB+ VP/J      NPl/V3+ . D/P NSg/Vg  NPr/VB+ NSg/I/C/Ddem+ VPt R       I/Ddem VB/C
> achieved accuracy in      the 93 – 95 % range    .
# VP/J     N🅪Sg+    NPr/J/P D   #  . #  . N🅪Sg/VB+ .
>
#
> Eugene Charniak points  out          in      Statistical techniques for natural language
# NPr+   ?        NPl/V3+ NSg/VB/J/R/P NPr/J/P J           NPl        C/P NSg/J+  N🅪Sg/VB+
> parsing ( 1997 ) that          merely assigning the most         common   tag     to each known word    and
# Nᴹ/Vg/J . #    . NSg/I/C/Ddem+ R      Nᴹ/Vg/J   D   NSg/I/J/R/Dq NSg/VB/J NSg/VB+ P  Dq   VPp/J NSg/VB+ VB/C
> the tag     " proper noun    " to all          unknowns will    approach 90 % accuracy because many
# D   NSg/VB+ . NSg/J  NSg/VB+ . P  NSg/I/J/C/Dq NPl/V3+  NPr/VXB NSg/VB+  #  . N🅪Sg+    C/P     NSg/I/J/Dq
> words   are unambiguous , and  many       others  only  rarely represent their less       - common
# NPl/V3+ VB  J           . VB/C NSg/I/J/Dq NPl/V3+ J/R/C R      VB        D$+   VB/J/R/C/P . NSg/VB/J
> parts  of speech   .
# NPl/V3 P  N🅪Sg/VB+ .
>
#
> CLAWS   pioneered the field  of HMM - based part     of speech   tagging but     was quite
# NPl/V3+ VP/J      D   NSg/VB P  VB  . VP/J  NSg/VB/J P  N🅪Sg/VB+ NSg/Vg  NSg/C/P VPt R
> expensive since it       enumerated all          possibilities . It       sometimes had to resort to
# J         C/P   NPr/ISg+ VP/J       NSg/I/J/C/Dq NPl+          . NPr/ISg+ R         VB  P  NSg/VB P
> backup methods when    there were    simply too many       options ( the Brown      Corpus
# NSg/J  NPl/V3+ NSg/I/C +     NSg/VPt R      W?  NSg/I/J/Dq NPl/V3  . D+  NPr🅪/VB/J+ NSg+
> contains a   case     with 17 ambiguous words  in      a    row     , and  there are words   such  as
# V3       D/P NPr🅪/VB+ P    #  J         NPl/V3 NPr/J/P D/P+ NSg/VB+ . VB/C +     VB  NPl/V3+ NSg/I NSg/R
> " still    " that          can     represent as    many       as    7 distinct parts  of speech   .
# . NSg/VB/J . NSg/I/C/Ddem+ NPr/VXB VB        NSg/R NSg/I/J/Dq NSg/R # VB/J     NPl/V3 P  N🅪Sg/VB+ .
>
#
> HMMs underlie the functioning of stochastic taggers and  are used in      various
# ?    VB       D   Nᴹ/Vg/J+    P  J          NPl     VB/C VB  VP/J NPr/J/P J
> algorithms one        of the most         widely used being       the bi    - directional inference
# NPl+       NSg/I/VB/J P  D   NSg/I/J/R/Dq R      VP/J N🅪Sg/Vg/J/C D   NSg/J . NSg/J       NSg+
> algorithm .
# NSg       .
>
#
> Dynamic programming methods
# NSg/J+  Nᴹ/Vg/J+    NPl/V3+
>
#
> In      1987 , Steven DeRose and  Kenneth W. Church   independently developed dynamic
# NPr/J/P #    . NPr+   ?      VB/C NPr+    ?  NPr🅪/VB+ R             VP/J      NSg/J
> programming algorithms to solve  the same problem in      vastly less       time       . Their
# Nᴹ/Vg/J+    NPl+       P  NSg/VB D   I/J  NSg/J+  NPr/J/P R      VB/J/R/C/P N🅪Sg/VB/J+ . D$+
> methods were    similar to the Viterbi algorithm known for some     time       in      other
# NPl/V3+ NSg/VPt NSg/J   P  D   ?       NSg       VPp/J C/P I/J/R/Dq N🅪Sg/VB/J+ NPr/J/P NSg/VB/J
> fields    . DeRose used a   table  of pairs   , while      Church   used a   table  of triples and  a
# NPrPl/V3+ . ?      VP/J D/P NSg/VB P  NPl/V3+ . NSg/VB/C/P NPr🅪/VB+ VP/J D/P NSg/VB P  NPl/V3  VB/C D/P
> method of estimating the values  for triples that          were    rare     or    nonexistent in      the
# NSg/VB P  Nᴹ/Vg/J    D   NPl/V3+ C/P NPl/V3  NSg/I/C/Ddem+ NSg/VPt NSg/VB/J NPr/C NSg/J       NPr/J/P D
> Brown     Corpus ( an  actual measurement of triple   probabilities would require a   much
# NPr🅪/VB/J NSg+   . D/P NSg/J  N🅪Sg        P  NSg/VB/J NPl+          VXB   NSg/VB  D/P NSg/I/J/R/Dq
> larger corpus ) . Both   methods achieved an  accuracy of over    95 % . DeRose's 1990
# JC     NSg+   . . I/C/Dq NPl/V3+ VP/J     D/P N🅪Sg+    P  NSg/J/P #  . . ?        #
> dissertation at    Brown     University included analyses     of the specific error   types   ,
# NSg+         NSg/P NPr🅪/VB/J NSg+       VP/J     NPl/V3/Au/Br P  D   NSg/J    NSg/VB+ NPl/V3+ .
> probabilities , and  other    related data  , and  replicated his     work     for Greek    , where
# NPl+          . VB/C NSg/VB/J J       N🅪Pl+ . VB/C VP/J       ISg/D$+ N🅪Sg/VB+ C/P NPr/VB/J . NSg/C
> it       proved similarly effective .
# NPr/ISg+ VP/J   R         NSg/J     .
>
#
> These   findings were    surprisingly disruptive to the field  of natural language
# I/Ddem+ NSg+     NSg/VPt R            J          P  D   NSg/VB P  NSg/J+  N🅪Sg/VB+
> processing . The accuracy reported was higher than the typical accuracy of very
# Nᴹ/Vg/J+   . D+  N🅪Sg+    VP/J     VPt NSg/JC C/P  D   NSg/J   N🅪Sg     P  J/R
> sophisticated algorithms that          integrated part     of speech   choice  with many       higher
# VP/J+         NPl+       NSg/I/C/Ddem+ VP/J       NSg/VB/J P  N🅪Sg/VB+ N🅪Sg/J+ P    NSg/I/J/Dq NSg/JC
> levels of linguistic analysis : syntax , morphology , semantics , and  so        on  . CLAWS   ,
# NPl/V3 P  J          N🅪Sg     . Nᴹ+    . Nᴹ+        . NPl+      . VB/C NSg/I/J/C J/P . NPl/V3+ .
> DeRose's and  Church's methods did fail     for some     of the known cases   where
# ?        VB/C NSg$     NPl/V3+ VPt NSg/VB/J C/P I/J/R/Dq P  D   VPp/J NPl/V3+ NSg/C
> semantics is  required , but     those  proved negligibly rare     . This   convinced many       in
# NPl+      VL3 VP/J     . NSg/C/P I/Ddem VP/J   R          NSg/VB/J . I/Ddem VP/J      NSg/I/J/Dq NPr/J/P
> the field   that          part      - of - speech   tagging could   usefully be      separated from the other
# D+  NSg/VB+ NSg/I/C/Ddem+ NSg/VB/J+ . P  . N🅪Sg/VB+ NSg/Vg  NSg/VXB R        NSg/VXB VP/J      P    D   NSg/VB/J
> levels of processing ; this    , in      turn   , simplified the theory and  practice of
# NPl/V3 P  Nᴹ/Vg/J+   . I/Ddem+ . NPr/J/P NSg/VB . VP/J       D   N🅪Sg   VB/C NSg/VB   P
> computerized language analysis and  encouraged researchers to find   ways to
# VP/J         N🅪Sg/VB+ N🅪Sg+    VB/C VP/J       NPl+        P  NSg/VB NPl+ P
> separate other    pieces  as    well     . Markov Models  became the standard method  for the
# NSg/VB/J NSg/VB/J NPl/V3+ NSg/R NSg/VB/J . NPr    NPl/V3+ VPt    D   NSg/J    NSg/VB+ C/P D
> part      - of - speech   assignment .
# NSg/VB/J+ . P  . N🅪Sg/VB+ NSg+       .
>
#
> Unsupervised taggers
# VB/J         NPl
>
#
> The methods already discussed involve working from a    pre       - existing corpus to
# D+  NPl/V3+ W?      VP/J      VB      Nᴹ/Vg/J P    D/P+ NSg/VB/P+ . Nᴹ/Vg/J  NSg+   P
> learn  tag     probabilities . It       is  , however , also possible to bootstrap using
# NSg/VB NSg/VB+ NPl+          . NPr/ISg+ VL3 . C       . R/C  NSg/J    P  NSg/VB    Nᴹ/Vg/J
> " unsupervised " tagging . Unsupervised tagging techniques use     an  untagged corpus
# . VB/J         . NSg/Vg  . VB/J         NSg/Vg  NPl+       N🅪Sg/VB D/P J        NSg+
> for their training data  and  produce the tagset by      induction . That          is  , they
# C/P D$+   Nᴹ/Vg/J+ N🅪Pl+ VB/C Nᴹ/VB   D   NSg    NSg/J/P N🅪Sg      . NSg/I/C/Ddem+ VL3 . IPl+
> observe patterns in      word    use     , and  derive part      - of - speech   categories themselves .
# NSg/VB  NPl/V3+  NPr/J/P NSg/VB+ N🅪Sg/VB . VB/C NSg/VB NSg/VB/J+ . P  . N🅪Sg/VB+ NPl+       IPl+       .
> For example , statistics readily reveal that          " the " , " a   " , and  " an  " occur in
# C/P NSg/VB+ . NPl/V3+    R       NSg/VB NSg/I/C/Ddem+ . D   . . . D/P . . VB/C . D/P . VB    NPr/J/P
> similar contexts , while      " eat " occurs in      very different ones    . With sufficient
# NSg/J+  NPl/V3+  . NSg/VB/C/P . VB  . V3     NPr/J/P J/R  NSg/J+    NPl/V3+ . P    J
> iteration , similarity classes of words   emerge that          are remarkably similar to
# N🅪Sg      . NSg        NPl/V3  P  NPl/V3+ NSg/VB NSg/I/C/Ddem+ VB  R          NSg/J   P
> those  human    linguists would expect ; and  the differences themselves sometimes
# I/Ddem NSg/VB/J NPl+      VXB   VB     . VB/C D   NPl/VB+     IPl+       R
> suggest valuable new      insights .
# VB      NSg/J    NSg/VB/J NPl+     .
>
#
> These   two  categories can     be      further subdivided into rule    - based , stochastic , and
# I/Ddem+ NSg+ NPl+       NPr/VXB NSg/VXB VB/J    VP/J       P    NSg/VB+ . VP/J  . J          . VB/C
> neural approaches .
# J      NPl/V3+    .
>
#
> Other    taggers and  methods
# NSg/VB/J NPl     VB/C NPl/V3+
>
#
> Some     current major    algorithms for part      - of - speech   tagging include the Viterbi
# I/J/R/Dq NSg/J   NPr/VB/J NPl        C/P NSg/VB/J+ . P  . N🅪Sg/VB+ NSg/Vg  NSg/VB  D   ?
> algorithm , Brill tagger , Constraint Grammar  , and  the Baum - Welch algorithm ( also
# NSg       . NSg/J NSg    . NSg+       N🅪Sg/VB+ . VB/C D   NPr  . ?     NSg       . R/C
> known as    the forward  - backward algorithm ) . Hidden Markov model     and  visible Markov
# VPp/J NSg/R D   NSg/VB/J . NSg/J    NSg       . . VB/J   NPr    NSg/VB/J+ VB/C J       NPr
> model     taggers can     both   be      implemented using   the Viterbi algorithm . The
# NSg/VB/J+ NPl     NPr/VXB I/C/Dq NSg/VXB VP/J        Nᴹ/Vg/J D   ?       NSg       . D+
> rule    - based Brill tagger is  unusual in      that         it       learns a   set      of rule    patterns , and
# NSg/VB+ . VP/J  NSg/J NSg    VL3 NSg/J   NPr/J/P NSg/I/C/Ddem NPr/ISg+ NPl/V3 D/P NPr/VB/J P  NSg/VB+ NPl/V3+  . VB/C
> then    applies those  patterns rather     than optimizing a   statistical quantity .
# NSg/J/C V3      I/Ddem NPl/V3+  NPr/VB/J/R C/P  Nᴹ/Vg/J    D/P J           N🅪Sg+    .
>
#
> Many        machine learning methods have    also been    applied to the problem of POS
# NSg/I/J/Dq+ NSg/VB+ Nᴹ/Vg/J+ NPl/V3+ NSg/VXB R/C  NSg/VPp VP/J    P  D   NSg/J   P  NSg+
> tagging . Methods such  as    SVM , maximum entropy classifier , perceptron , and
# NSg/Vg  . NPl/V3+ NSg/I NSg/R ?   . NSg/J   NSg     NSg        . NSg        . VB/C
> nearest - neighbor     have    all          been    tried , and  most         can     achieve accuracy above
# JS      . NSg/VB/J/Am+ NSg/VXB NSg/I/J/C/Dq NSg/VPp VP/J  . VB/C NSg/I/J/R/Dq NPr/VXB VB      N🅪Sg+    NSg/J/P
> 95 % . [ citation needed ]
# #  . . . NSg+     VP/J   .
>
#
> A   direct comparison of several methods is  reported ( with references ) at    the ACL
# D/P VB/J   NSg        P  J/Dq+   NPl/V3+ VL3 VP/J     . P    NPl/V3+    . NSg/P D   NSg
> Wiki    . This    comparison uses   the Penn tag     set      on  some     of the Penn Treebank data  ,
# NSg/VB+ . I/Ddem+ NSg+       NPl/V3 D+  NPr+ NSg/VB+ NPr/VB/J J/P I/J/R/Dq P  D   NPr+ ?        N🅪Pl+ .
> so        the results are directly comparable . However , many       significant taggers are
# NSg/I/J/C D   NPl/V3+ VB  R/C      NSg/J      . C       . NSg/I/J/Dq NSg/J       NPl     VB
> not   included ( perhaps because of the labor          involved in      reconfiguring them     for
# NSg/C VP/J     . NSg/R   C/P     P  D   NPr🅪/VB/Am/Au+ VP/J     NPr/J/P Nᴹ/Vg/J       NSg/IPl+ C/P
> this   particular dataset ) . Thus , it       should not   be      assumed that         the results
# I/Ddem NSg/J      NSg     . . NSg  . NPr/ISg+ VXB    NSg/C NSg/VXB VP/J    NSg/I/C/Ddem D+  NPl/V3+
> reported here    are the best       that          can     be      achieved with a    given       approach ; nor   even
# VP/J     NSg/J/R VB  D   NPr/VXB/JS NSg/I/C/Ddem+ NPr/VXB NSg/VXB VP/J     P    D/P+ NSg/VB/J/P+ NSg/VB+  . NSg/C NSg/VB/J
> the best        that          have    been    achieved with a    given       approach .
# D+  NPr/VXB/JS+ NSg/I/C/Ddem+ NSg/VXB NSg/VPp VP/J     P    D/P+ NSg/VB/J/P+ NSg/VB+  .
>
#
> In      2014 , a    paper      reporting using   the structure regularization method for
# NPr/J/P #    . D/P+ N🅪Sg/VB/J+ Nᴹ/Vg/J   Nᴹ/Vg/J D   N🅪Sg/VB+  N🅪Sg           NSg/VB C/P
> part      - of - speech   tagging , achieving 97.36 % on  a   standard benchmark dataset .
# NSg/VB/J+ . P  . N🅪Sg/VB+ NSg/Vg  . Nᴹ/Vg/J   #     . J/P D/P NSg/J    NSg/VB    NSg     .
