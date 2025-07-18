import React, { useState } from 'react';
import { useRole } from '../../context/RoleContext';
import { useMarket } from '../../context/MarketContext';
import { useNavigate } from 'react-router-dom';

const MarketCreator = () => {
  const { currentRole, hasPermission } = useRole();
  const { createMarket, loading } = useMarket();
  const navigate = useNavigate();

  const [formData, setFormData] = useState({
    question: '',
    outcomes: ['Yes', 'No'],
    settlementTime: '',
    description: ''
  });

  const [errors, setErrors] = useState({});

  // Redirect if not oracle
  if (!hasPermission('create_market')) {
    return (
      <div className="bg-red-400 border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
        <h2 className="text-2xl font-bold mb-4 font-['Space_Grotesk']">❌ ACCESS DENIED</h2>
        <p className="text-lg mb-4">Only oracles can create markets.</p>
        <button 
          onClick={() => navigate('/roles')}
          className="bg-white border-2 border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] px-4 py-2 font-bold hover:transform hover:translate-x-1 hover:translate-y-1 transition-all duration-200"
        >
          SWITCH TO ORACLE
        </button>
      </div>
    );
  }

  const handleInputChange = (e) => {
    const { name, value } = e.target;
    setFormData(prev => ({
      ...prev,
      [name]: value
    }));
    
    // Clear error when user starts typing
    if (errors[name]) {
      setErrors(prev => ({
        ...prev,
        [name]: ''
      }));
    }
  };

  const handleOutcomeChange = (index, value) => {
    const newOutcomes = [...formData.outcomes];
    newOutcomes[index] = value;
    setFormData(prev => ({
      ...prev,
      outcomes: newOutcomes
    }));
  };

  const addOutcome = () => {
    if (formData.outcomes.length < 5) {
      setFormData(prev => ({
        ...prev,
        outcomes: [...prev.outcomes, '']
      }));
    }
  };

  const removeOutcome = (index) => {
    if (formData.outcomes.length > 2) {
      const newOutcomes = formData.outcomes.filter((_, i) => i !== index);
      setFormData(prev => ({
        ...prev,
        outcomes: newOutcomes
      }));
    }
  };

  const validateForm = () => {
    const newErrors = {};

    if (!formData.question.trim()) {
      newErrors.question = 'Question is required';
    }

    if (!formData.settlementTime) {
      newErrors.settlementTime = 'Settlement time is required';
    } else {
      const settleTime = new Date(formData.settlementTime);
      if (settleTime <= new Date()) {
        newErrors.settlementTime = 'Settlement time must be in the future';
      }
    }

    if (formData.outcomes.some(outcome => !outcome.trim())) {
      newErrors.outcomes = 'All outcomes must be filled';
    }

    if (formData.outcomes.length < 2) {
      newErrors.outcomes = 'At least 2 outcomes are required';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    
    if (!validateForm()) {
      return;
    }

    const settlementTime = new Date(formData.settlementTime).getTime();
    const market = await createMarket(
      formData.question,
      formData.outcomes.filter(o => o.trim()),
      settlementTime
    );

    if (market) {
      navigate(`/betting/${market.id}`);
    }
  };

  const getMinDateTime = () => {
    const now = new Date();
    now.setMinutes(now.getMinutes() + 30); // At least 30 minutes from now
    return now.toISOString().slice(0, 16);
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-white border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
        <h2 className="text-2xl font-bold mb-2 font-['Space_Grotesk']">🏦 CREATE MARKET</h2>
        <p className="text-gray-600">
          Create a new prediction market as an oracle
        </p>
      </div>

      {/* Form */}
      <div className="bg-white border-4 border-black shadow-[8px_8px_0px_0px_rgba(0,0,0,1)] p-6">
        <form onSubmit={handleSubmit} className="space-y-6">
          {/* Question */}
          <div>
            <label className="block text-lg font-bold mb-2 font-['Space_Grotesk']">
              Market Question *
            </label>
            <input
              type="text"
              name="question"
              value={formData.question}
              onChange={handleInputChange}
              placeholder="e.g., Will Bitcoin reach $100k by end of 2024?"
              className="w-full p-3 border-2 border-black font-mono text-lg focus:outline-none focus:ring-2 focus:ring-orange-400"
              maxLength={200}
            />
            {errors.question && (
              <p className="text-red-600 text-sm mt-1">{errors.question}</p>
            )}
            <p className="text-sm text-gray-500 mt-1">
              {formData.question.length}/200 characters
            </p>
          </div>

          {/* Description */}
          <div>
            <label className="block text-lg font-bold mb-2 font-['Space_Grotesk']">
              Description (Optional)
            </label>
            <textarea
              name="description"
              value={formData.description}
              onChange={handleInputChange}
              placeholder="Additional context or rules for the market..."
              className="w-full p-3 border-2 border-black font-mono text-sm focus:outline-none focus:ring-2 focus:ring-orange-400"
              rows="3"
              maxLength={500}
            />
            <p className="text-sm text-gray-500 mt-1">
              {formData.description.length}/500 characters
            </p>
          </div>

          {/* Outcomes */}
          <div>
            <label className="block text-lg font-bold mb-2 font-['Space_Grotesk']">
              Possible Outcomes *
            </label>
            {formData.outcomes.map((outcome, index) => (
              <div key={index} className="flex items-center mb-2">
                <input
                  type="text"
                  value={outcome}
                  onChange={(e) => handleOutcomeChange(index, e.target.value)}
                  placeholder={`Outcome ${index + 1}`}
                  className="flex-1 p-3 border-2 border-black font-mono text-lg focus:outline-none focus:ring-2 focus:ring-orange-400"
                  maxLength={50}
                />
                {formData.outcomes.length > 2 && (
                  <button
                    type="button"
                    onClick={() => removeOutcome(index)}
                    className="ml-2 bg-red-400 border-2 border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] px-3 py-3 font-bold hover:transform hover:translate-x-1 hover:translate-y-1 transition-all duration-200"
                  >
                    ❌
                  </button>
                )}
              </div>
            ))}
            {formData.outcomes.length < 5 && (
              <button
                type="button"
                onClick={addOutcome}
                className="bg-green-400 border-2 border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] px-4 py-2 font-bold hover:transform hover:translate-x-1 hover:translate-y-1 transition-all duration-200"
              >
                + ADD OUTCOME
              </button>
            )}
            {errors.outcomes && (
              <p className="text-red-600 text-sm mt-1">{errors.outcomes}</p>
            )}
          </div>

          {/* Settlement Time */}
          <div>
            <label className="block text-lg font-bold mb-2 font-['Space_Grotesk']">
              Settlement Time *
            </label>
            <input
              type="datetime-local"
              name="settlementTime"
              value={formData.settlementTime}
              onChange={handleInputChange}
              min={getMinDateTime()}
              className="w-full p-3 border-2 border-black font-mono text-lg focus:outline-none focus:ring-2 focus:ring-orange-400"
            />
            {errors.settlementTime && (
              <p className="text-red-600 text-sm mt-1">{errors.settlementTime}</p>
            )}
            <p className="text-sm text-gray-500 mt-1">
              When the market will be settled and the outcome determined
            </p>
          </div>

          {/* Preview */}
          <div className="bg-gray-100 border-2 border-black p-4">
            <h3 className="text-lg font-bold mb-2 font-['Space_Grotesk']">📋 PREVIEW</h3>
            <div className="space-y-2">
              <div><strong>Question:</strong> {formData.question || 'No question set'}</div>
              <div><strong>Outcomes:</strong> {formData.outcomes.filter(o => o.trim()).join(', ') || 'No outcomes set'}</div>
              <div><strong>Settlement:</strong> {formData.settlementTime ? new Date(formData.settlementTime).toLocaleString() : 'No time set'}</div>
            </div>
          </div>

          {/* Submit Button */}
          <div className="flex items-center justify-between">
            <button
              type="button"
              onClick={() => navigate('/')}
              className="bg-gray-400 border-2 border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] px-6 py-3 font-bold hover:transform hover:translate-x-1 hover:translate-y-1 transition-all duration-200"
            >
              CANCEL
            </button>
            <button
              type="submit"
              disabled={loading}
              className="bg-orange-400 border-2 border-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)] px-6 py-3 font-bold hover:transform hover:translate-x-1 hover:translate-y-1 transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loading ? 'CREATING...' : 'CREATE MARKET'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

export default MarketCreator;