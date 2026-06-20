# fuzzy_dict
Dictionary based fuzzy filter


It is demonstrated as the first stage of this demo.

Using a fast hash function based on bits representing the presence of classes of letters and buckets of valid strings to match, we severely restrict the search space using an algorithm that is very fast, and it can incrementally widen the search if necessary through incremental permutation. Mostly a week-end itch scratcher, use it if it is useful to you.

Not on crate.io but is open source. If anyone wants to polish the code and upload it, I am ok with it 👍.

Good bits in hte paper.
