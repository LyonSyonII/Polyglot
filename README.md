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


### Structs
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
Also, in this case one-liners are supported, using "->" next to the condition.
    
    if condition
        // something
    
    if condition    -> // something
    elif condition  -> // something
    else            -> // something

You can chain conditions with the "&&" and "||" operators

    if condition && 

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


### Functions (indentation not necessary)
    fn void
        // something
    end
    
    fn void_args(a: int, b: num, ...)
        // something
    end
    
    fn return: int
        // something
        ret 5
    end
    
    fn return_args_fn(a: int, b: num, ...): string
        // something
        "hello"
    end


You can return multiple things with tuples, order must be preserved when returning
    
    fn return_multiple: int, num, char
        25, 3.14, 'a'
    end


To take a tuple as an argument, use parenthesis in between the type annotation
    
    fn tuple_args(tuple: (int, num, char))
        // something
    end


You can also name each argument on a tuple
    
    fn tuple_named_args(tuple: (a: int, b: num, c: char))
        // something
    end

Tuples can be destructured when passed as args, so a more idiomatic way to write the above expression would be
    
    fn tuple_idiomatic_args(a: int, b: num, c: char)
        // something
    end
    
    tuple = 89, 99.99, 'z'
    tuple_idiomatic_args(tuple)


To return early, use the "ret" keyword. The last line in a function will always be treated as an implicit return
    
    fn return_early(a: num, b: num): int
        if a > b -> ret a - b
        else if b > a -> ret b - a
        else -> 0
    end


## Basic operations
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