(ns clean
  "It is actually possible to document a ns.
  It's a nice place to describe the purpose of the namespace and maybe even
  the overall conventions used. Note how _not_ indenting the docstring makes
  it easier for tooling to display it correctly.")

;;;; Section Comment/Heading

;;; Foo...
;;; Bar...
;;; Baz...

;; good
(defn foo
  "This funtion doesn't do much."
  []
  nil)

;; bad
(defn bar
  ^{:doc "This function doesn't do much."}
  []
  nil)

;; good
(defn qzuf-number
  "Computes the [Qzuf number](https://wikipedia.org/qzuf) of the `coll`.
  Supported options in `opts`:

  | key           | description |
  | --------------|-------------|
  | `:finite-uni?`| Assume finite universe; default: `false`
  | `:complex?`   | If OK to return a [complex number](https://en.wikipedia.org/wiki/Complex_number); default: `false`
  | `:timeout`    | Throw an exception if the computation doesn't finish within `:timeout` milliseconds; default: `nil`

  Example:
  ```clojure
  (when (neg? (qzuf-number [1 2 3] {:finite-uni? true}))
    (throw (RuntimeException. \"Error in the Universe!\")))
  ```"
  [coll opts]
  nil)

(defprotocol MyProtocol
  "MyProtocol docstring"
  (foo [this x y z]
    "foo docstring")
  (bar [this]
    "bar docstring"))

;; good
(defn some-fun
  []
  ;; FIXME: This has crashed occasionally since v1.2.3. It may
  ;;        be related to the BarBazUtil upgrade. (xz 13-1-31)
  #_(baz))

;;;; Frob Grovel

;;; This section of code has some important implications:
;;;   1. Foo.
;;;   2. Bar.
;;;   3. Baz.

(defn fnord [zarquon]
  ;; If zob, then veeblefitz.
  (quux zot
        mumble             ; Zibblefrotz.
        frotz))

(defn foo [x]
  x ; I'm a line/code fragment comment.
  )

;;; I'm a top-level comment.
;;; I live outside any definition.

(defn foo [])

(def ^{:deprecated "0.5"} foo
  "Use `bar` instead."
  42)
