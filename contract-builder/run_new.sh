#!/bin/sh

. env.sh

docker run \
     --mount type=bind,source=$HOST_DIR,target=/host \
     --cap-add=SYS_PTRACE --security-opt seccomp=unconfined \
     --name=$CONTAINER_NAME \
     -i -t $IMAGE_NAME \
     /bin/bash

