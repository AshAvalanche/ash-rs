#!/bin/bash

envsubst <./blueprint.yaml | cargo run -- console blueprint apply -
