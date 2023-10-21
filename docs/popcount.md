# Switch to Popcount for Bitset Size

Since I learned that `int.count_ones()` is a single CPU instruction,
I have wanted to remove the internal size from the bitsets and compare.
Now that I have the automated solver, It's time to give it a shot.
Whether that will be a good comparison is another story.

## Cost of Tracking Size

Updating the internal size requires more operations for each change:

1. Check if the bit is already set.
   1. Shift the bit into position.
   2. Perform the test with bit-AND.
2. Set the bit.
   1. Shift the bit into position.
      This could be avoided by inlining the check above.
   2. Perform the set with bit-OR.
   3. Increment the size with addition.

Reading the size is not free, either. Since it's stored in the higher bits,
it requires a shift to extract the value. No mask is required since there
are no bits higher than the size.

Compare these with

- Setting a bit requires a single shift and bit-OR.
  - No need to check if the bit is already set.
  - No need to update the size.
- Reading the size requires a single popcount CPU operation.
  - If the bit shift is one clock cycle, this is the same cost.

## ChatGPT

According to ChatGPT, I'm on the right track.

> Using the processor's count-ones operation (also known as population count or popcount)
> to determine the number of set bits in your bitset can indeed be faster than storing
> and updating the count separately, especially on modern x64 processors. The popcount
> instruction is specifically designed for this purpose and is highly optimized in hardware
> for efficient bit counting.
> 
> Here are a few reasons why using the processor's popcount instruction can be faster:
> 
> 1. Hardware optimization: Modern x64 processors have dedicated hardware support
>    for population count operations. This means that the CPU can perform these operations 
>    very quickly, often in a single cycle.
> 
> 2. Reduced memory access: If you store the count separately, you'll need to read and update
>    that count in memory each time you modify the bitset. This involves memory access,
>    which is typically slower than performing operations entirely within the CPU's registers.
> 
> 3. Fewer instructions: The popcount operation typically involves fewer instructions than manually
>    updating the count using shifts and additions, which can lead to shorter execution time.
> 
> 4. Parallelism: In some cases, the CPU may be able to perform popcount operations in parallel
>    with other instructions, which can further improve performance.
> 
> So, if your primary concern is performance and you are working on a modern x64 processor,
> it's a good idea to use the popcount instruction to count the set bits in your bitset.
> This approach can lead to faster execution and more efficient use of hardware resources.

The saved bit-AND and addition should save 2-3 clock cycles per bit set/clear
with no cost increase for reading the size. Sadly, none of the stored sizes
increase the size of the bitsets, so other will be no memory savings.

- The 9 bit coord/known sets would still need 16 bits.
- The 81 bit cell set would still need 128 bits.

## Results

- Solving the small set of easy puzzles went from ~70ms to ~60ms, almost 15% faster.
- Solving the larger set of just-17s went from ~26.1s to ~22.8s, almost 13% faster.
