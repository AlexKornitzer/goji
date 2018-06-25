# 0.2.2

* added sprints interfaces [#24](https://github.com/softprops/goji/pull/24)
* added boarders interfaces [#21](https://github.com/softprops/goji/pull/21)
* added agile api [#20](https://github.com/softprops/goji/pull/20)

# 0.2.1

* updated issue and attachment interfaces

# 0.2.0

* replace hyper client with reqwest

# 0.1.1

* expanded search interface with and `iter` method that implements an `Iterator` over `Issues`
* changed `SearchListOptionsBuilder#max` to `max_results` be more consistent with the underlying api
* introduced `Error::Unauthorized` to handle invalid credentials with more grace
* replaced usage of `u32` with `u64` for a more consistent interface
* renamed `TransitionTrigger` to `TransitionTriggerOptions` for a more consistent api

# 0.1.0

* initial release