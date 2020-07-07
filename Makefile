all:
	@echo Use \`cargo\` to build.
	@echo Use \`make submission\` to submit.

submission:
	git co submission
	git merge master
	git co master
	git push origin submission
