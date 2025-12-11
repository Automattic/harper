"""
    test(x::Number, y::String, z...)

This is a doctring for function `test`. `x test y test z test...` calls this function with
several arguments, like `*(x, y, z...)`.

# Arguments
- `x::Number`: a number
- `y::String`: a string
- `z::Vector`: a vector

# Examples
```jldoctest
julia> a = [1 2; 3 4]
2×2 Matrix{Int64}:
 1  2
 3  4
```
"""
function test(x::Number, y::String, z::Vector ...)
    somethig = 1 # This is a inline comment with too mistakes.
    return somethig
end

"Here's a one-line docstring weeth weerd spelleeng."
x = 2

@doc raw"""
This is is annother wei too document zjierb.
"""
f(x) = x
