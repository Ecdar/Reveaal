# PR - Review Guide
- PR's should be made as a draft at first and reviewed internally by a group member before its sent out for external review.
- Published PR's should be reviewed within a day.

## Reviewing a PR
- Ensure the new functionality described in the PR is implemented and working.
- Run ```Cargo bench``` on main as well as the PR branch. Compare the results to check if performance is intact.
- Check if the code is properly documented. This means I/O and panic criteria should be documented as well as what it does if it is public
- All tests should pass and if there is new functionality it should be tested. It is however up to the reviewers to judge if the tests are enough.
- 2 External Reviewers should approve the PR before its considered done.