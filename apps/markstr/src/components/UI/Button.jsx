import React from 'react';

const Button = ({ 
  children, 
  onClick, 
  disabled = false, 
  variant = 'primary', 
  size = 'medium', 
  className = '',
  ...props 
}) => {
  const baseStyles = 'border-2 border-black font-bold transition-all duration-200 hover:transform hover:translate-x-1 hover:translate-y-1 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:transform-none';
  
  const variants = {
    primary: 'bg-orange-400 shadow-[4px_4px_0px_0px_rgba(0,0,0,1)]',
    secondary: 'bg-cyan-400 shadow-[4px_4px_0px_0px_rgba(0,0,0,1)]',
    success: 'bg-green-400 shadow-[4px_4px_0px_0px_rgba(0,0,0,1)]',
    warning: 'bg-yellow-400 shadow-[4px_4px_0px_0px_rgba(0,0,0,1)]',
    danger: 'bg-red-400 shadow-[4px_4px_0px_0px_rgba(0,0,0,1)]',
    gray: 'bg-gray-400 shadow-[4px_4px_0px_0px_rgba(0,0,0,1)]',
    white: 'bg-white shadow-[4px_4px_0px_0px_rgba(0,0,0,1)]',
  };

  const sizes = {
    small: 'px-3 py-1 text-sm',
    medium: 'px-4 py-2',
    large: 'px-6 py-3 text-lg',
  };

  return (
    <button
      onClick={onClick}
      disabled={disabled}
      className={`${baseStyles} ${variants[variant]} ${sizes[size]} ${className}`}
      {...props}
    >
      {children}
    </button>
  );
};

export default Button;