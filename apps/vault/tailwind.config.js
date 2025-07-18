export default {
  content: ['./index.html', './src/**/*.{js,jsx,ts,tsx}'],
  theme: {
    extend: {
      colors: {
        primary: '#FF6B00',
        secondary: '#00D4FF',
        tertiary: '#FFE500',
        success: '#00FF88',
        danger: '#FF0066',
        dark: '#000000',
        light: '#FFFFFF',
        gray: {
          100: '#F5F5F5',
          200: '#E5E5E5',
          300: '#D4D4D4',
          400: '#A3A3A3',
          500: '#737373',
          600: '#525252',
          700: '#404040',
          800: '#262626',
          900: '#171717',
        },
      },
      fontFamily: {
        grotesk: ['Space Grotesk', 'monospace'],
      },
      boxShadow: {
        brutal: '8px 8px 0px rgba(0,0,0,1)',
        'brutal-sm': '4px 4px 0px rgba(0,0,0,1)',
        'brutal-lg': '12px 12px 0px rgba(0,0,0,1)',
      },
    },
  },
  plugins: [],
} 