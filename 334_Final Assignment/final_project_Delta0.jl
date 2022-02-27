module FP

using Distributions
using Printf

# Show floats to 2 digits after the decimal point
Base.show(io::IO, f::Float64) = @printf(io, "%2.2f", f)

Δ = 0.0     # propagation delay upper bound
h = 0.12    # honest mining rate
a = 0.08    # adversarial mining rate
λ = h + a   # total mining rate

K = 30 # confirmation depth

println("Simulation begins.\n")
println(
    "confirmation depth=",
    K,
    "  max delay=",
    Δ,
    "  honest mining rate=",
    h,
    "  adversarial mining rate=",
    a,
)

nmax = round(Int, K * 3 * λ / a)   # In principle the first violation can occur in the arbitrary future.
# However, the probability that it first occurs at a height much larger than K is extremely small.  Here we only simulate up to about 3K adversarial blocks.

ntrials = 1000

violation = 0.0

for nt = 1:ntrials
    xx = rand(Exponential(1.0 / λ), nmax)   # exponential inter-mining times for all blocks
    tt = cumsum(xx)   # the (cumulative) mining times
    bb = rand(Bernoulli(h / λ), nmax)   # each block is honest with probability h/λ
    bb[1] = true    # We assume block 1 is always honest - which is the target of the attack.

    tth = tt[bb]        # honest block mining times
    tta = tt[.!bb]      # adversarial block mining times

    len = min(length(tth), length(tta))

    if any(tth[K:len] .> tta[K:len])
        global violation += 1
    end

    if nt == 1  # print an example of
        println("\nThe following are the mining times in trial #1")
        for n = 1:nmax
            print(bb[n] ? "honest" : "advers")
            println(" block mined at ", tt[n], "    after interval ", xx[n])
            # println( bb[n], " ", xx[n], " ", tt[n] )
        end
        println(
            "\nThe mining times of the first ",
            len,
            " honest blocks:\n",
            tth[1:len],
        )
        println(
            "The mining times of the first ",
            len,
            " adversarial blocks:\n",
            tta[1:len],
        )
        println(
            "When the adversarial is ahead in this trial (1)?  ",
            tth[1:len] .> tta[1:len],
        )
    end
end

println(
    "\nThe probability of block 1's safety violation is about ",
    100.0 * violation / ntrials,
    "% based on ",
    ntrials,
    " trials.",
)
println("Simulation ends.")

end
