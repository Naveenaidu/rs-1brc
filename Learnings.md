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