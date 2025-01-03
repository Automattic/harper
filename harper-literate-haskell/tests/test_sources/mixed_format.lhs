Sourced from https://wiki.haskell.org/Literate_programming.

In Bird-style you have to leave a blnk before the code.

> fact :: Integer -> Integer
> fact 0 = 1
> fact n = n * fact (n-1)

And you have to leave a blnk line after the code as well.

And the definition of the following function
would totally screw up my program, so I'm not
definining it:

\begin{code}
main :: IO ()
main = print "just an example"
\end{code}

Seee?
