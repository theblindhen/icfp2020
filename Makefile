all:
	@echo Use \`cargo\` to build.
	@echo Use \`make submission\` to submit.

submission:
	git checkout submission
	git merge master
	git checkout master
	git push origin submission
