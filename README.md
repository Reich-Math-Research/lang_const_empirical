# lang_const_empirical
A repo for analyzing the Lang conjecture on linear forms in logarithms, specifically investigating the coefficient that the conjecture asserts does exist, but nobody really knows what it is.

## Introduction to Lang's Conjecture

### Baker's Theorem: Logs of (Certain) Algebraic Numbers Don't Add to Zero, Ever

Baker's theorem is a well known and extremely powerful result in analytic number theory. A linear form in logarithms is an expression of the form

$$\Lambda:=b_1\log\ \alpha_1+b_2\log\ \alpha_2\dotsc+b_n\log\ \alpha_n.$$

Baker's theorem says that if $\alpha_1,\dotsc,\alpha_n$ don't depend on each other in a multiplicative way and $b_i$ are all integers, then 

$$|\Lambda|\geq\exp\left(-C\left(\log\ B\right)^n\right),$$

where $B=\max\{|b_1|,\dotsc,|b_n|\}$ and $-C$ was computable based of the algebraic heights of $\alpha_i$ (which is basically the measure of how complicated $\alpha_i$ is). There's only one problem: Baker's bound has both a really bad constant and a really bad growth rate. For example, certain linear forms corresponded to values with $x\leq 3$. However, Baker's theorem proved that $x\leq e^{e^{e^{e^{730}}}}$. Ouch!

### Lang's Conjecture: How Good Can Baker Get?

Lang's conjecture basically says that the best Baker's theorem can get is this:

$$|\Lambda|\geq \frac{C(\epsilon, \alpha_1,\dotsc\alpha_n)}{B^{n-1+\epsilon}}$$

for some $\epsilon>0$ and a computable constant $C$, when you fix $\alpha_i$ and vary $b_i$. Sounds simple, right? It definitely isn't.

## The "Constant Problem"

There's one issue though: Lang's conjecture doesn't actually tell you what $C$ is supposed to look like. And worse, nobody really knows, since Lang's conjecture is already really, really hard to prove. Adding a constant term to prove to that mess makes the problem basically impossible to tackle, even with modern math tools.

But there's still hope: authors have suggested that $C$ is the inverse of the heights of $\alpha_i$ all multiplied together. Some have suggested the inverse of the heights summed together. Some have suggested $e^{-c/\epsilon}$ for some $c$ dependent on $\alpha_i$. But these are all just guesses, with vague intuition and no real evidence behind them.

Baker's theorem was proven in the 60s, and Lang's conjecture was made in 1966. Back then, computation was, well, pretty bad, when even getting 3 digits of precision on a logarithm was a monumental task. They didn't have much in the way of supporting evidence to figure out what $C$ was. But now, we do.

## But How?

First, we will construct the linear form we want to examine in terms of $\alpha_1,\alpha_2$ (Note: our current architecture only supports two term logarithms for quadratic algebraic numbers due to difficulty implementing the height function for arbitrary algebraic numbers). Then we set $b_{\text{min}}$ and $b_{\text{max}}$. Then, we compute the $b_2$ value for when

$$\Lambda=b_1\log(\alpha_1) + b_2\log(\alpha_2)$$

is closest to zero. Denote this $b_2$ value the min of $|\Lambda|$. Since if Lang's conjecture holds for the closest value, it holds for all the other values, this is rigorous. Then, set

$$C_{B_{\text{max}}}(\epsilon)=\min(|\Lambda|)\cdot B^{n-1+\epsilon},$$

so the final value of $C(\epsilon)$ should be

$$\lim_{B_{\text{max}}\to\infty}C_{B_{\text{max}}}(\epsilon).$$

This is effectively the "tightest possible constant" that Lang's conjecture could allow.

**Note:** All scripts allow arbitrary precision. In general, we will default to 2000 bits or greater due to the tiny sizes of $\Lambda$.

## Constant "Regimes"

Of course, $\min(|\Lambda|)$ up to $B_{\text{max}}$ is a discrete quantity, controlled by one single pair of $b_1,b_2$. We denote a "regime" as a part of the graph of $C(\epsilon)$ where $\min(|\Lambda|)$ is a constant, so the only thing varying is $B^{n-1+\epsilon}$. This creates a tension between minimizing $B$, favoring small $b_i$, and minimizing $|\Lambda|$, favoring large $b_i$ (See the next section). The optimal point is exactly what creates regimes that prevent the curve from being a simple exponential.

![A $C(\epsilon)$ curve, with regime changes as the "corners".](lang_results/a1_(sqrt(2)+1)∕2_a2_(1+sqrt(3))∕2_b1_1_to_10000000/a1_(sqrt(2)+1)∕2_a2_(1+sqrt(3))∕2_b1_1_to_10000000.png "Demonstration of a $C(\epsilon)$ Curve")

## Connection to Diophantine Approximation

If $\Lambda\approx0$, then $b_1\log(\alpha_1)+b_2\log(\alpha_2)$, which means that $\frac{b_1}{b_2}\approx-\frac{\log(\alpha_2)}{\log(\alpha_1)}$. This is exactly a problem of **Diophantine Approximation**, specifically approximating the transcendental $-\frac{\log(\alpha_2)}{\log(\alpha_1)}$. Exceptionally good rational numbers that approximate $\frac{\log(\alpha_2)}{\log(\alpha_1)}$ are called convergents of $\frac{\log(\alpha_2)}{\log(\alpha_1)}$. It is plain then, that larger $b_i$ should make $|\Lambda|$ smaller, and this is the central tension of Lang's conjecture.

The precise reason why Baker's bounds are so bad is because they ignore the structure of the convergents, and DA in general. They act as a "sledgehammer", pretending as if $b_i$ and $\alpha_i$ are out to get them. In reality, such unison between $b_i$ and $\alpha_i$ is extremely rare, and this is the exact structure that Lang's conjecture takes into account while Baker's bound doesn't.

## Mission Statement

In this repository, we build the framework to numerically investigate hundreds of linear forms in logarithms, building enormous datasets, to figure out: **"what really is $C$?"** In addition, we provide the tools to graphically analyze these enormous datasets.

- **Hypothesis 1:** $C(\epsilon)=\exp\left(\frac{1}{\epsilon\prod h(\alpha_i)}\right)$
- **Hypothesis 2:** $C(\epsilon)=\exp\left(\frac{1}{\epsilon\left(\sum h(\alpha_i)\right)}\right)$
- **Hypothesis 3:** $C(\epsilon)=\exp\left(\frac{\epsilon}{\prod h(\alpha_i)}\right)$
- **Hypothesis 4:** $C(\epsilon)=\exp\left(\frac{\epsilon}{\sum h(\alpha_i)}\right)$
