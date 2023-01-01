# setup basic location variables
export sh_loc="$(dirname -- "$0")/runner.sh"
export nu_loc="$(dirname -- "$0")/runner.nu"
export hopper_loc=$PWD/target/release/hopper
cat $sh_loc > ./temp_sh
cat $nu_loc > ./temp_nu
sed -i "s|HOPPERCMD|$hopper_loc|g" ./temp_sh ./temp_nu

# add runner function to .zshrc if needed
if [ -f ~/.zshrc ] && ! grep -q "hop()" ~/.zshrc; then
    echo "[info] adding .zshrc runner..."
    cat ./temp_sh >> ~/.zshrc
fi

# add runner function to .bashrc if needed
if [ -f ~/.bashrc ] && ! grep -q "hop()" ~/.bashrc; then
    echo "[info] adding .bashrc runner..."
    cat ./temp_sh >> ~/.bashrc
fi

# add runner function to nushell env.nu if needed
if [ -f ~/.config/nushell/env.nu ] && ! grep -q "def-env hop" ~/.config/nushell/env.nu; then
    echo "[info] adding nushell runner..."
    cat ./temp_nu >> ~/.config/nushell/env.nu
fi

rm ./temp_sh ./temp_nu
