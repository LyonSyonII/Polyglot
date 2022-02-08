### Variable Initialization
Variable types will be inferred.
    
    var integer = 5
    var number = 5.5
    var boolean = true|false
    var character = 'a'
    var string = "abc"
    var tuple = "abc", 5

Access the elements of an anonymous tuple with their index.
    
    var other_string = tuple.0
    var other_int = tuple.1

### Variable declaration (without initialization)
If a variable is not initialized, the type must be annotated explicitely.
    
    var integer: int
    var number: num 
    var boolean: bool
    var character: char
    var string: str
    var tuple: str, int

### Named Tuples (Structs)
Structs are basically enhanced tuples with member names.
    
    var person = age: 52, name: "Alex"
    var num = person.age

You can define a struct with type anotations, just like any variable.
    
    var person: (age: num, name: str)

It is useful to declare your structs as types for reusability and understandability.
    
    type Person = age: num, name: str
    var person: Person = 52, "Alex"     
    // If the explicit type anotation is not made, "person" will be treated as an anonymous tuple
    print(person.age)

### Enums
Enums are a way to tell the compiler that a variable can only have a number of values.

    type StrBool = "true" | "false"

    // Correct
    var a: StrBool = "true"
    
    // Compiler error
    var a: StrBool = "1"

Enums can also be named.  
Names will be treated as aliases for the real value.
    
    type StrBool = T -> "true" | F -> "false"
    var a: StrBool = StrBool.T
    // a = "true"

You can use these names to check for conditions.

    if a = StrBool.T => print("a is true")
    else             => print("a is false")

Or [match](#match) each value

### Lists
    var int_list = [5, 6, 7, 8]
    var num_list = [5, 5.5, 8.8]
    
    var str_list: [str]
    var tuple_list: [(str, bool)]
    tuple_list = [("true", true), ("false", false)]

To add things to a list, use the + operator.
    
    var int_list: [int]
    int_list += 5
    int_list = int_list + 5

You can also add multiple lists, but they must be of the same type.
    
    var list = [1, 2, 3]
    var other = [4, 5, 6]
    list += other                   
    // list = [1, 2, 3, 4, 5, 6]

To remove an element from a list, use the - operator.
    
    var list = [1, 2, 3]
    list -= 1                       
    // list = [2, 3]

You can substract multiple lists, basically removing the first intersection.
    
    var list = [1, 2, 3]
    var other = [2, 3]
    list -= other                   
    // list = [1]

The remove operator will only remove the first occurrence, if you want to remove all of them, use the -- operator.
    
    var list = [1, 2, 3, 1, 2, 3]
    list --= 1                      
    // list = [2, 3, 2, 3]
    var other = [1, 2]
    list --= other                  
    // list = [3, 3]

To access a specific element on a list, use [index].
    
    var list = ["hello", "world"]
    print(list[1])                  
    // output -> "world"

If an out-of-bounds error is detected (trying to access an element that isn't there) the program will be terminated at runtime.
    
    print(list[2])                  
    // output -> ERROR: Out-of-bounds in line X, "list[2]".

You can define the starting size of a list.
    
    var list: [int, 3]

### Dictionaries
Dictionaries act like lists, but instead of using the index of insertion to access each element, you use a custom defined one.
    
    var dictionary = [ "true" -> true, "false" -> false ]
    print(dictionary["true"])
    // output -> true

To declare an empty dictionary you must annotate its types explicitely
    
    var dictionary: [str -> bool]

> Depending on the flavor of Polyglot you're using, the implementation of the Dictionaries will change drastically, so don't assume constant lookup.  
> If the language in question has a native dictionary, then it will be used.

### Control flow
#### If / Else If / Else
If some condition is met, execute the code inside the `if`.  
If the condition is false, execute the code inside the `elif` (else if).  
If all the conditions above are false, execute the code inside the `else`.
    
    if condition
        // something
    end
    
    if condition
        // something
    elif condition
        // something else
    else
         // other things
    end

The "end" keyword can be omitted when only one expression is inside the if/elseif/else.
Also, in this case one-liners are supported, using "=>" next to the condition.
    
    // Omit "end"    
    if condition
        // something
    
    // One-liners
    if condition    => // something
    elif condition  => // something
    else            => // something
    
    // NOT VALID
    if condition // something

You can chain conditions with the "&&" and "||" operators
    
    // Check if a > 0 AND a < 5
    if a > 0 && a < 5
        print("a is in the (0, 5) range")
    end
    
    // Check if a < 0 OR a > 0
    if a < 0 || a > 0
        print("a is not 0")
    end

#### Match
The match pattern allows to check if a variable has certain compile-time values.  
You can indicate an "if any of these" using "_". 
    
    var number = 5 
    match number
        1 => print("one")
        2 => print("two")
        3 => print("three")
        4 => print("four")
        5 => print("five")
        _ => print("number not defined")

On most languages it is basically identical to an if / else if / else
    
    var number = 5
    if   number = 1 => print("one")
    elif number = 2 => print("two")
    elif number = 3 => print("three")
    elif number = 4 => print("four")
    elif number = 5 => print("five")
    else            => print("number not defined")

> Do not assume that match will be translated as a lookup table, it is flavor dependent.  
> If the language has a similar implementation (`switch` in C/C++, `match` in Rust) then match will be transpiled into this implementation.

You can write multiple lines on each block, but then the pattern must finish with an "end".
    
    match number
        1 => number += 1
             print("one")
        2 => number += 2
             print("two")
        ...
    end

You can match against multiple values.
    
    match number
        1 | 2 | 3 => print("one, two or three")
        _         => print("other numbers") 

### Loops
    while condition
        // something
    end
    
    for i in 0..100
        // something
    end
    
    for item in list
        // something
    end

### Functions
Indentation is not necessary, but it is strongly recommended.
    
    fn void
        // something
    end
    
    fn void_args(a: int, b: num, ...)
        // something
    end
    
To return something from a function, first anotate the type of the return.
    
    fn return: int

Then use "ret"
    fn return: int
        // something
        ret 5
    end
    
    fn return_args_fn(a: int, b: num, ...): string
        // something
        "hello"
    end


You can return multiple things, order must be preserved when returning.
    
    fn return_multiple: int, num, char
        25, 3.14, 'a'
    end

To return early, use the "ret" keyword. The last line in a function will always be treated as an implicit return.
    
    fn return_early(a: num, b: num): int
        if a > b -> ret a - b
        else if b > a -> ret b - a
        else -> 0
    end


To take a tuple as an argument, use parenthesis in between the type annotation.
    
    fn tuple_args(tuple: (int, num, char))
        // something
    end


You can also pass named tuples.
    
    fn tuple_named_args(struct: (a: int, b: num, c: char))
        // something
    end

Tuples can be destructured when passed as args, so a more idiomatic way to write the above expression would be:
    
    fn tuple_idiomatic_args(a: int, b: num, c: char)
        // something
    end
    
    tuple = 89, 99.99, 'z'
    tuple_idiomatic_args(tuple)

It is recommended to create a type definition for Structs that you'll be using frequently.
    
    type Animal = name: str, species: Species

### Basic operations
    var number = 10
    var ops: num
    
    // Addition
    ops = 5 + 6
    ops = number + 15
    ops += 65
    
    // Substraction
    ops = 5 - 6
    ops = 15 - number
    ops -= 65
    
    // Multiplication
    ops = 5 * 6
    ops = number * 15
    ops *= 65
    
    // Division
    ops = 5 / 6
    ops = number / 15
    ops /= 65
    
    // Exponentiation
    ops = 5^6
    ops = number ^ 15
    ops ^= 65

### Comparations

    // Equal
    if a = b

    // Not equal
    if a != b
    
    // Smaller
    if a < b
    
    // Smaller or equal
    if a <= b

    // Larger
    if a > b

    // Larger or equal
    if a >= b

    // Negation
    if !a
