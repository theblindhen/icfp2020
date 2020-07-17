all:
	@echo Use \`cargo\` to build.
	@echo Use \`make submission\` to submit.

.PHONY: docker-build
docker-build: 
	docker build -t icfp20 .

.PHONY: docker-run
docker-run: docker-build
	docker run --rm icfp20

.PHONY: docker-shell
docker-shell: docker-build
	docker run --rm -it --entrypoint /bin/bash icfp20

submission:
	git checkout submission
	git pull
	git merge master
	git checkout master
	git push origin submission
