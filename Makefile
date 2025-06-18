out/parser_host/index.json: \
	$(shell git ls-files images/parser_host src)
	$(call build,parser_host)

out/parser_app/index.json: \
	$(shell git ls-files images/parser_app src)
	$(call build,parser_app)

# Import environment variable
GITHUB_TOKEN := $(shell echo $$GITHUB_TOKEN)

# Debug what we got
$(info GITHUB_TOKEN imported: '$(GITHUB_TOKEN)')
$(info GITHUB_TOKEN length: $(shell echo '$(GITHUB_TOKEN)' | wc -c))

# Base docker args
DOCKER_BUILD_ARGS = --build-arg VERSION=$(VERSION)

# Check if token exists and add secret arg
ifneq ($(strip $(GITHUB_TOKEN)),)
    $(info Adding GitHub token secret to build args)
    SECRET_FILE := $(shell mktemp -u --tmpdir docker-secret-XXXXXXXXXX)
    DOCKER_BUILD_ARGS += --secret id=github_token,src=$(SECRET_FILE)
    WRITE_SECRET_CMD = @echo "Writing token to $(SECRET_FILE)" && echo "$(GITHUB_TOKEN)" > $(SECRET_FILE)
    REMOVE_SECRET_CMD = @echo "Removing $(SECRET_FILE)" && rm -f $(SECRET_FILE)
else
    $(info No GitHub token found, skipping secret)
    WRITE_SECRET_CMD = @echo "No GitHub token provided"
    REMOVE_SECRET_CMD = @true
endif

$(info Final DOCKER_BUILD_ARGS: $(DOCKER_BUILD_ARGS))

.PHONY: debug-token
debug-token:
    @echo "GITHUB_TOKEN from env: '$$GITHUB_TOKEN'"
    @echo "GITHUB_TOKEN in make: '$(GITHUB_TOKEN)'"
    @echo "Length: $(shell echo '$(GITHUB_TOKEN)' | wc -c)"


.PHONY: non-oci-docker-images
non-oci-docker-images:
	$(WRITE_SECRET_CMD)
	docker buildx build $(DOCKER_BUILD_ARGS) --load --tag anchorageoss-visualsign-parser/parser_app -f images/parser_app/Containerfile .
	$(REMOVE_SECRET_CMD)

define build_context
$$( \
	mkdir -p out; \
	self=$(1); \
	for each in $$(find out/ -maxdepth 2 -name index.json); do \
    	package=$$(basename $$(dirname $${each})); \
    	if [ "$${package}" = "$${self}" ]; then continue; fi; \
    	printf -- ' --build-context %s=oci-layout://./out/%s' "$${package}" "$${package}"; \
	done; \
)
endef

.PHONY: debug-make-vars
debug-make-vars:
	@echo "=== MAKE VARIABLE DEBUG ==="
	@echo "Raw env GITHUB_TOKEN: '$$GITHUB_TOKEN'"
	@echo "Make GITHUB_TOKEN: '$(GITHUB_TOKEN)'"
	@echo "GITHUB_TOKEN empty check: '$(if $(GITHUB_TOKEN),NOT EMPTY,EMPTY)'"
	@echo "ifneq condition: '$(shell if [ -n '$(GITHUB_TOKEN)' ]; then echo TRUE; else echo FALSE; fi)'"
	@echo "DOCKER_BUILD_ARGS: '$(DOCKER_BUILD_ARGS)'"
	@echo "SECRET_FILE: '$(SECRET_FILE)'"
	@echo "=== END DEBUG ==="

,:=,
define build
	$(eval NAME := $(1))
	$(eval TYPE := $(if $(2),$(2),dir))
	$(eval REGISTRY := anchorageoss-visualsign-parser)
	$(eval PLATFORM := linux/amd64)
	$(WRITE_SECRET_CMD) && \
	DOCKER_BUILDKIT=1 \
	SOURCE_DATE_EPOCH=1 \
	BUILDKIT_MULTIPLATFORM=1 \
	docker build \
		$(DOCKER_BUILD_ARGS) \
		--tag $(REGISTRY)/$(NAME) \
		--progress=plain \
		--platform=$(PLATFORM) \
		--label "org.opencontainers.image.source=https://github.com/anchorageoss/visualsign-parser" \
		$(if $(filter common,$(NAME)),,$(call build_context,$(1))) \
		$(if $(filter 1,$(NOCACHE)),--no-cache) \
		--output "\
			type=oci,\
			$(if $(filter dir,$(TYPE)),tar=false$(,)) \
			rewrite-timestamp=true,\
			force-compression=true,\
			name=$(NAME),\
			$(if $(filter tar,$(TYPE)),dest=$@") \
			$(if $(filter dir,$(TYPE)),dest=out/$(NAME)") \
		-f images/$(NAME)/Containerfile \
		. && \
	$(REMOVE_SECRET_CMD)
endef
