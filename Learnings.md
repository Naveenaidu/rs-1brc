1. Harder to round floating point number. Had to switch to Decimal
    * Why is that the case?
    * There are different rounding off stratergy -> Figure them out
    * https://users.rust-lang.org/t/why-doesnt-round-have-an-argument-for-digits/100688/24

2. Flamegraph vs Traces
    * https://www.polarsignals.com/blog/posts/2023/03/28/how-to-read-icicle-and-flame-graphs

Question:
1. What is Box in rust?
    * Why did i need to use `Box<dyn error::Error>` in my main() function
    * TLDR: "dyn" allows you to store in a Box a mix of Apples and Oranges,
      because they all implement the same trait of Fruit, which is what your Box
      is using as a type constraint, instead of just a generic type. This is
      because Generic allows any ONE of Apple OR Orange, but not both:
    * Ref: https://stackoverflow.com/questions/50650070/what-does-dyn-mean-in-a-type

2. When reading large files, always store them as raw bytes vector of u8:
    ```rs
    let mut file_content = Vec::new();
    let mut file = std::fs::File::open(&args.file).expect("Failed to open file");
    std::io::Read::read_to_end(&mut file, &mut file_content)
        .expect("Failed to read file content");
    ```

3. Problem with using `String::from_utf8` to convert Vec<u8> to string
    * WHAT: It's taking up the entire memory, even the swap space -> The program gets killed
    * WHY:
        * 
    * How to fix:
        Ended up using a unsafe version which skips the check

4. Problem with `cp`
    * Created the measurement.txt file from in gunna 1brc repo, it showed 1B lines when counted with wc
    * copied it to my repo and 2 lines got removed. WHY? 
    * Any bug with `cp`?

5. Using `String:from_utf8` sometimes reads the file correctly and other times it doesn't
    * Why is that the case
    * whenever I use print!() statements then it usually ends up reading correctly!
    * Looks like there is a problem with my Ram, see thread with Shraddha on this

6. Do not ever use print!() in hot path
    * Using print in the hot path ended up taking a long time
    * This is because, print will take locks from OS and this will increase time
    * This was the reason the first iteration was slow.
