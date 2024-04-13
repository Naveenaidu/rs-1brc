There are different ways to read the file, we might have to compare their performance.

Testing:
-> Comparing the diff will not work
-> Read through the .out file spit by program and the test file and then compare 
   the hashmap.

-> A program, which takes a file and gives us back a hashmap

TODO:
1. Write the values to file -> With one round off digit
2. Write the code for test case to compare the values with the *.out file
    -> We can just read the *.out file into a hashmap
    -> We can then compare the hashmap
3. Code to write the result to the file


TODO:

tests failing for measurements-20.txt -> FIX THIS -> DONE

----------------------------

30/03/24

* Test naive solution
* Have a flamegraph OR a trace.

IDEA: We can write a blog about how to setup tracing infra and code in RUST
    * Not enough documentation about it
    * How to setup global tracer
    * How to have nested span, multiple span etc

-------

10/04/24

* Analyze the flamegraph and see where the issue is happening
* The time with second appraoch is around 20 minutes
* Maybe change the Hash function
* How about instead of Buffer read - we read the entire file at once?

-----------

11/04/24

* Use read_line() instead of lines()
* Instead of using Vec() in read_line() directly use next() -> This will avoid the allocation into
  vector
* 

--------------

13/04/24

* Not using a convert to vector has reduced the time by a lot. Looks like allocation takes a long time.

Ideas:

How to speed up read_line:
* parse seems to take a long time - Instead of using a lot of extra verification -> We might not need it
* Create our own Decimal parser

* read_lines() vs read_line()
  -> https://users.rust-lang.org/t/why-using-the-read-lines-iterator-is-much-slower-than-using-read-line/92815/7

* use a hashmap<&static str, String>
