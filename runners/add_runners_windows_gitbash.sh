# setup basic location variables
export sh_loc="$(dirname -- "$0")/gitbash_runner.sh"
export hopper_loc=$PWD/target/release/hopper
cat $sh_loc > ./temp_sh
sed -i "s|HOPPERCMD|$hopper_loc|g" ./temp_sh

# add runner function to .zshrc if needed
if [ -f ~/.bash_profile ] && ! grep -q "hop()" ~/.bash_profile; then
    echo "[info] adding git-bash runner..."
    cat ./temp_sh >> ~/.bash_profile
else
    echo "[info] git-bash runner skipped due to existing/missing/conflicting configuration"
fi
