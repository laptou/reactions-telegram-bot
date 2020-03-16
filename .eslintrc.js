module.exports = {
    root: true,
    parser: '@typescript-eslint/parser',
    env: {
        browser: true,
        node: true,
        es2017: true
    },
    parserOptions: {
        tsconfigRootDir: __dirname,
        project: ['./tsconfig.json'],
        sourceType: 'module',
        ecmaFeatures: {
            jsx: true //allows for the parsing of JSX
        }
    },
    plugins: [
        '@typescript-eslint',
    ],
    extends: [
        'eslint:recommended',
        'plugin:@typescript-eslint/eslint-recommended',
        'plugin:@typescript-eslint/recommended',
        'plugin:@typescript-eslint/recommended-requiring-type-checking'
    ],
    rules: {
        '@typescript-eslint/explicit-function-return-type': ['off'],
        '@typescript-eslint/no-var-requires': ['off'],
        '@typescript-eslint/no-non-null-assertion': ['off'],
        '@typescript-eslint/no-explicit-any': ['off'],
        '@typescript-eslint/member-ordering': ['error', {
            default: [
                // Index signature
                'signature',
              
                // Static
                'private-static-field',
                'protected-static-field',
                'public-static-field',
                'private-static-method',
                'protected-static-method',
                'public-static-method',

                'private-instance-field',
                'protected-instance-field',
                'public-instance-field',
              
                // Constructors
                'private-constructor',
                'protected-constructor',
                'public-constructor',
              
                // Methods
                'private-instance-method',
                'protected-instance-method',
                'public-instance-method',

                // Abstract
                'private-abstract-field',
                'protected-abstract-field',
                'public-abstract-field',
                'private-abstract-method',
                'protected-abstract-method',
                'public-abstract-method',
              ]
        }],
        '@typescript-eslint/explicit-member-accessibility': ['error', { accessibility: 'explicit' }],
        '@typescript-eslint/prefer-readonly': ['warn'],
        '@typescript-eslint/unbound-method': ['warn'],
        'sort-imports': ['warn'],

        // spacing rules
        'block-spacing': ['error', 'always'],
        'key-spacing': ['error'],
        'keyword-spacing': ['error', { before: true, after: true }],
        'object-curly-spacing': ['error', 'always'],
        'array-bracket-spacing': ['error', 'never'],
        'computed-property-spacing': ['error', 'never'],
        'space-in-parens': ['error', 'never'],
        'space-before-blocks': ['error', 'always'],
        'no-whitespace-before-property': ['error'],
        'func-call-spacing': ['error', 'never'],
        'indent': ['error', 2],
        
        // comma rules
        'comma-dangle': ['error', 'never'],
        'comma-spacing': ['error', { before: false, after: true }],
        'brace-style': ['error', '1tbs', { allowSingleLine: true }],

        // quote rules
        'jsx-quotes': ['error', 'prefer-single'],
        'quote-props': ['error', 'consistent-as-needed'],
        'quotes': ['error', 'single'],

        // other rules
        'semi': ['error', 'always'],
        'no-warning-comments': ['warn'],
        'no-debugger': ['off']
    },
    settings: {
    }
};