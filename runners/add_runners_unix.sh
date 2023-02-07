# setup basic location variables
export sh_loc="$(dirname -- "$0")/runner.sh"
export nu_loc="$(dirname -- "$0")/runner.nu"
export hopper_loc=$PWD/target/release/hop🐇
cat $sh_loc > ./temp_sh
cat $nu_loc > ./temp_nu
sed -i "s|__HOPPERCMD__|$hopper_loc|g" ./temp_sh ./temp_nu

# add runner function to .zshrc if needed
if [ -f ~/.zshrc ] && ! grep -q "hp()" ~/.zshrc; then
    echo "[info] adding zsh runner..."
    cat ./temp_sh >> ~/.zshrc
else
    echo "[info] zsh runner skipped due to existing/missing/conflicting configuration"
fi

# add runner function to .bashrc if needed
if [ -f ~/.bashrc ] && ! grep -q "hp()" ~/.bashrc; then
    echo "[info] adding bash runner..."
    cat ./temp_sh >> ~/.bashrc
else
    echo "[info] bash runner skipped due to existing/missing/conflicting configuration"
fi

# add runner function to nushell env.nu if needed
if [ -f ~/.config/nushell/env.nu ] && ! grep -q "def-env hp" ~/.config/nushell/env.nu; then
    echo "[info] adding nushell runner..."
    cat ./temp_nu >> ~/.config/nushell/env.nu
else
    echo "[info] nushell runner skipped due to existing/missing/conflicting configuration"
fi

rm ./temp_sh ./temp_nu
