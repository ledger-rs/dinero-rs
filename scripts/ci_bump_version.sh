# $1 - semver string
# $2 - level to incr {release,minor,major} - release by default
function bump_version() { 
    local RE='[^0-9]*\([0-9]*\)[.]\([0-9]*\)[.]\([0-9]*\)\([0-9A-Za-z-]*\)'
    major=`echo $1 | sed -e "s#$RE#\1#"`
    minor=`echo $1 | sed -e "s#$RE#\2#"`
    release=`echo $1 | sed -e "s#$RE#\3#"`
    # patch=`echo $1 | sed -e "s#$RE#\4#"`
    
    release=0
    minor=$((minor+1))

    echo "$major.$minor.$release"
}

version=$(grep -E "version = \"([0-9]+\.[0-9]+.[0-9]+)\"" Cargo.toml | grep -Eo -m 1 "[0-9]+\.[0-9]+.[0-9]+")
bumped=$(bump_version ${version})
message="Bump from $version to $bumped"
commit_message="[ci skip] ${message}"

# Update Cargo.toml
line_number=$(grep -En "version = \"([0-9]+\.[0-9]+.[0-9]+)\"" Cargo.toml | grep -Eo -m 1 "[0-9]+" | head -n 1)

sed -i "${line_number}s/.*/version = \"${bumped}-dev\"/" Cargo.toml 

# Update Changelog
sed -i "3i## [${bumped}] - xxx" CHANGELOG.md

echo ${commit_message}
