import React from 'react';

const Card = ({ children, className = '', color = 'white', ...props }) => {
  const colorClasses = {
    white: 'bg-white',
    orange: 'bg-orange-400',
    cyan: 'bg-cyan-400',
    yellow: 'bg-yellow-400',
    green: 'bg-green-400',
    red: 'bg-red-400',
    purple: 'bg-purple-400',
    gray: 'bg-gray-400',
  };

  return (
    <div 
      className={`${colorClasses[color]} border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] ${className}`}
      {...props}
    >
      {children}
    </div>
  );
};

export default Card;